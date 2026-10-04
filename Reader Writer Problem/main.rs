// Readers-writers with semaphores, instrumented, plus a live visualization server.
//
// Usage (keep main.rs and rw_viz.html in the same folder):
//   rustc main.rs -o main
//   ./main [readers] [writers] [iterations] [--basic] [--port=8080]
//   then open http://127.0.0.1:8080
//
// Defaults: 3 readers, 2 writers, 3 iterations each, port 8080.
// --basic disables the turnstile (reader-priority version, writers can starve).
//
// The program runs the experiment, records every step as a JSON event, and serves
// them to the page over Server-Sent Events (/events). Events are buffered, so the
// page can connect before, during or after the run, and "Replay" gets the full history.
// std only, no crates. Press Ctrl+C to stop the server.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering::SeqCst};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

const PAGE: &str = include_str!("rw_viz.html");

/// Counting semaphore built from a Mutex + Condvar (std has no semaphore).
struct Semaphore {
    count: Mutex<usize>,
    cv: Condvar,
}

impl Semaphore {
    fn new(n: usize) -> Self {
        Semaphore { count: Mutex::new(n), cv: Condvar::new() }
    }

    fn wait(&self) {
        let mut c = self.count.lock().unwrap();
        while *c == 0 {
            c = self.cv.wait(c).unwrap();
        }
        *c -= 1;
    }

    fn signal(&self) {
        let mut c = self.count.lock().unwrap();
        *c += 1;
        self.cv.notify_one();
    }
}

struct Shared {
    fair: bool,

    // synchronization
    turnstile: Semaphore,   // fairness gate (only used when fair)
    wrt: Semaphore,         // exclusive access for writers
    read_count: Mutex<u32>, // lock protecting the reader count

    // the shared resource
    resource: AtomicU32,

    // instrumentation
    active_readers: AtomicU32,
    active_writers: AtomicU32,
    max_readers: AtomicU32,
    violations: AtomicU32,
    total_reads: AtomicU32,
    total_writes: AtomicU32,
    reader_wait_sum: AtomicU64,
    reader_wait_max: AtomicU64,
    writer_wait_sum: AtomicU64,
    writer_wait_max: AtomicU64,
    start: Instant,

    // event stream for the visualization
    events: Mutex<Vec<String>>,
    ev_cv: Condvar,
}

impl Shared {
    fn log(&self, color: &str, who: &str, msg: &str) {
        // a single println! is one atomic write, so lines never interleave
        println!(
            "[{:>5}ms] {}{:<9} {:<14}{} readers={} writers={}",
            self.start.elapsed().as_millis(),
            color,
            who,
            msg,
            RESET,
            self.active_readers.load(SeqCst),
            self.active_writers.load(SeqCst),
        );
    }

    fn violation(&self, who: &str, what: &str) {
        self.violations.fetch_add(1, SeqCst);
        eprintln!("{BOLD}{RED}INVARIANT VIOLATED by {who}: {what}{RESET}");
        self.emit(&format!("\"e\":\"viol\",\"who\":\"{who}\",\"msg\":\"{what}\""));
    }

    /// Append one JSON event (stamped with ms since start) and wake the streamers.
    fn emit(&self, body: &str) {
        let t = self.start.elapsed().as_secs_f64() * 1000.0;
        let json = format!("{{\"t\":{t:.1},{body}}}");
        self.events.lock().unwrap().push(json);
        self.ev_cv.notify_all();
    }

    /// Thread event: kind `e`, thread type `k` ('R' or 'W'), id, plus extra JSON fields
    /// (each starting with a comma).
    fn ev(&self, e: &str, k: char, id: u32, extra: &str) {
        self.emit(&format!("\"e\":\"{e}\",\"k\":\"{k}\",\"id\":{id}{extra}"));
    }
}

fn reader(id: u32, s: Arc<Shared>, iters: u32) {
    let who = format!("Reader {id}");
    for _ in 0..iters {
        let arrived = Instant::now();
        s.ev("arrive", 'R', id, "");
        s.log(GREEN, &who, "arrived");

        // turnstile: pass straight through, but block if a writer is waiting
        if s.fair {
            s.turnstile.wait();
            s.turnstile.signal();
            s.ev("stage", 'R', id, ",\"s\":\"lock\"");
        }

        // entry section
        {
            let mut rc = s.read_count.lock().unwrap();
            *rc += 1;
            if *rc == 1 {
                // first reader holds the read_count lock while it waits for wrt
                s.ev("stage", 'R', id, ",\"s\":\"wrt\",\"h\":\"lock\"");
                s.wrt.wait(); // first reader locks out writers
            }
        }

        let waited = arrived.elapsed().as_millis() as u64;
        s.reader_wait_sum.fetch_add(waited, SeqCst);
        s.reader_wait_max.fetch_max(waited, SeqCst);

        // critical section
        let n = s.active_readers.fetch_add(1, SeqCst) + 1;
        s.max_readers.fetch_max(n, SeqCst);
        if s.active_writers.load(SeqCst) != 0 {
            s.violation(&who, "reading while a writer is active");
        }
        s.ev("enter", 'R', id, &format!(",\"w\":{waited}"));
        s.log(GREEN, &who, &format!("entered ({waited}ms wait)"));

        let v = s.resource.load(SeqCst);
        thread::sleep(Duration::from_millis(100));

        if s.active_writers.load(SeqCst) != 0 {
            s.violation(&who, "a writer entered during my read");
        }
        s.active_readers.fetch_sub(1, SeqCst);
        s.total_reads.fetch_add(1, SeqCst);
        s.ev("leave", 'R', id, &format!(",\"v\":{v}"));
        s.log(GREEN, &who, &format!("left (read {v})"));

        // exit section
        {
            let mut rc = s.read_count.lock().unwrap();
            *rc -= 1;
            if *rc == 0 {
                s.wrt.signal(); // last reader lets writers in
            }
        }

        thread::sleep(Duration::from_millis(50));
    }
    s.ev("done", 'R', id, "");
}

fn writer(id: u32, s: Arc<Shared>, iters: u32) {
    let who = format!("Writer {id}");
    for _ in 0..iters {
        let arrived = Instant::now();
        s.ev("arrive", 'W', id, "");
        s.log(RED, &who, "arrived");

        if s.fair {
            s.turnstile.wait(); // close the gate to new readers
            s.ev("stage", 'W', id, ",\"s\":\"wrt\",\"h\":\"turnstile\"");
        }
        s.wrt.wait(); // wait for readers to drain / other writer to finish
        if s.fair {
            s.turnstile.signal(); // reopen the gate; we already hold wrt
        }

        let waited = arrived.elapsed().as_millis() as u64;
        s.writer_wait_sum.fetch_add(waited, SeqCst);
        s.writer_wait_max.fetch_max(waited, SeqCst);

        // critical section
        let w = s.active_writers.fetch_add(1, SeqCst) + 1;
        if w != 1 {
            s.violation(&who, "more than one writer active");
        }
        if s.active_readers.load(SeqCst) != 0 {
            s.violation(&who, "writing while readers are active");
        }
        s.ev("enter", 'W', id, &format!(",\"w\":{waited}"));
        s.log(RED, &who, &format!("entered ({waited}ms wait)"));

        let v = s.resource.fetch_add(1, SeqCst) + 1;
        thread::sleep(Duration::from_millis(150));

        s.active_writers.fetch_sub(1, SeqCst);
        s.total_writes.fetch_add(1, SeqCst);
        s.ev("leave", 'W', id, &format!(",\"v\":{v}"));
        s.log(RED, &who, &format!("left (wrote {v})"));

        s.wrt.signal();

        thread::sleep(Duration::from_millis(80));
    }
    s.ev("done", 'W', id, "");
}

// ---------- tiny HTTP server (std only) ----------

fn handle(mut stream: TcpStream, s: Arc<Shared>) {
    let mut buf = [0u8; 2048];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    let req = String::from_utf8_lossy(&buf[..n]);
    let path = req.split_whitespace().nth(1).unwrap_or("/");
    match path {
        "/" | "/index.html" => {
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                PAGE.len()
            );
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(PAGE.as_bytes());
        }
        "/events" => stream_events(stream, s),
        _ => {
            let _ = stream.write_all(
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            );
        }
    }
}

/// Server-Sent Events: replay everything recorded so far, then follow new events.
fn stream_events(mut stream: TcpStream, s: Arc<Shared>) {
    let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n";
    if stream.write_all(head.as_bytes()).is_err() {
        return;
    }
    let mut idx = 0;
    loop {
        let batch: Vec<String> = {
            let mut g = s.events.lock().unwrap();
            while idx >= g.len() {
                g = s.ev_cv.wait(g).unwrap();
            }
            g[idx..].to_vec()
        };
        idx += batch.len();
        for e in &batch {
            if write!(stream, "data: {e}\n\n").is_err() {
                return;
            }
        }
        if stream.flush().is_err() {
            return;
        }
        if batch.last().map_or(false, |e| e.contains("\"e\":\"end\"")) {
            return; // run finished; closing ends this stream
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let fair = !args.iter().any(|a| a == "--basic");
    let nums: Vec<u32> = args.iter().filter_map(|a| a.parse().ok()).collect();
    let n_readers = *nums.get(0).unwrap_or(&3);
    let n_writers = *nums.get(1).unwrap_or(&2);
    let iters = *nums.get(2).unwrap_or(&3);
    let port: u16 = args
        .iter()
        .find_map(|a| a.strip_prefix("--port="))
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    println!(
        "{BOLD}Mode: {} | {n_readers} readers, {n_writers} writers, {iters} iterations each{RESET}\n",
        if fair { "fair (turnstile)" } else { "basic (reader priority)" }
    );

    let shared = Arc::new(Shared {
        fair,
        turnstile: Semaphore::new(1),
        wrt: Semaphore::new(1),
        read_count: Mutex::new(0),
        resource: AtomicU32::new(0),
        active_readers: AtomicU32::new(0),
        active_writers: AtomicU32::new(0),
        max_readers: AtomicU32::new(0),
        violations: AtomicU32::new(0),
        total_reads: AtomicU32::new(0),
        total_writes: AtomicU32::new(0),
        reader_wait_sum: AtomicU64::new(0),
        reader_wait_max: AtomicU64::new(0),
        writer_wait_sum: AtomicU64::new(0),
        writer_wait_max: AtomicU64::new(0),
        start: Instant::now(),
        events: Mutex::new(Vec::new()),
        ev_cv: Condvar::new(),
    });

    // start the web server first so the page can connect at any time
    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap_or_else(|e| {
        eprintln!("cannot listen on port {port}: {e} (try --port=8081)");
        std::process::exit(1);
    });
    println!("Visualization: http://127.0.0.1:{port}  (events are buffered, open it any time)\n");
    {
        let srv = Arc::clone(&shared);
        thread::spawn(move || {
            for conn in listener.incoming() {
                if let Ok(stream) = conn {
                    let s2 = Arc::clone(&srv);
                    thread::spawn(move || handle(stream, s2));
                }
            }
        });
    }

    shared.emit(&format!(
        "\"e\":\"config\",\"fair\":{fair},\"readers\":{n_readers},\"writers\":{n_writers},\"iters\":{iters}"
    ));

    let mut handles = Vec::new();
    for id in 1..=n_readers {
        let s = Arc::clone(&shared);
        handles.push(thread::spawn(move || reader(id, s, iters)));
    }
    for id in 1..=n_writers {
        let s = Arc::clone(&shared);
        handles.push(thread::spawn(move || writer(id, s, iters)));
    }
    for h in handles {
        h.join().unwrap();
    }

    // summary
    let total_reads = shared.total_reads.load(SeqCst);
    let total_writes = shared.total_writes.load(SeqCst);
    let reads = (total_reads as u64).max(1);
    let writes = (total_writes as u64).max(1);
    let violations = shared.violations.load(SeqCst);
    let total_ms = shared.start.elapsed().as_millis();
    let max_readers = shared.max_readers.load(SeqCst);
    let r_avg = shared.reader_wait_sum.load(SeqCst) / reads;
    let r_max = shared.reader_wait_max.load(SeqCst);
    let w_avg = shared.writer_wait_sum.load(SeqCst) / writes;
    let w_max = shared.writer_wait_max.load(SeqCst);
    let fin = shared.resource.load(SeqCst);

    println!("\n{BOLD}=== Summary ==={RESET}");
    println!("Total time            : {total_ms} ms");
    println!("Total reads / writes  : {total_reads} / {total_writes}");
    println!("Max concurrent readers: {max_readers}");
    println!("Reader wait avg / max : {r_avg} ms / {r_max} ms");
    println!("Writer wait avg / max : {w_avg} ms / {w_max} ms");
    println!("Final value           : {fin}");
    if violations == 0 {
        println!("{GREEN}{BOLD}Invariants: OK (no violations){RESET}");
    } else {
        println!("{RED}{BOLD}Invariants: {violations} VIOLATION(S){RESET}");
    }

    shared.emit(&format!(
        "\"e\":\"end\",\"time\":{total_ms},\"reads\":{total_reads},\"writes\":{total_writes},\"maxr\":{max_readers},\"ra\":{r_avg},\"rm\":{r_max},\"wa\":{w_avg},\"wm\":{w_max},\"fin\":{fin},\"viol\":{violations}"
    ));

    println!("\nRun complete. Still serving http://127.0.0.1:{port} (Ctrl+C to stop).");
    loop {
        thread::park();
    }
}