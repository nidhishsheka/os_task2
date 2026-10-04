# 🍽️ Dining Philosophers Problem --- Rust

A Rust implementation of the classic **Dining Philosophers Problem**,
developed as part of an Operating Systems assignment.

The project demonstrates how multiple concurrent threads can safely
share limited resources while avoiding **deadlock** and reducing the
possibility of **starvation**.

------------------------------------------------------------------------

## 📌 Problem

The Dining Philosophers Problem consists of philosophers sitting around
a table with one chopstick between each pair.

Each philosopher repeatedly:

**Think → Get Hungry → Wait → Eat → Release Resources**

The challenge is to allow philosophers to eat concurrently without
creating deadlocks or unfair resource allocation.

This implementation uses **4 philosophers and 4 chopsticks**.

------------------------------------------------------------------------

## 💡 Solution

A **FIFO-based waiter mechanism** is used to control access to the
shared resources.

When a philosopher becomes hungry, they join a waiting queue. The waiter
allows a philosopher to eat only when the required conditions are
satisfied.

This prevents philosophers from independently grabbing one resource and
waiting indefinitely for another --- avoiding the classic circular-wait
deadlock.

The FIFO queue also provides a fair ordering mechanism for philosophers
waiting for their turn.

------------------------------------------------------------------------

## 🔧 Technologies & Concepts

**Language:** Rust

The implementation uses Rust's concurrency primitives:

  Component       Purpose
  --------------- ----------------------------------------------
  `std::thread`   Creates philosopher threads
  `Arc`           Shares synchronization state between threads
  `Mutex`         Protects shared state
  `Condvar`       Makes threads wait efficiently
  `VecDeque`      Maintains the FIFO waiting queue

### Operating System Concepts

-   Multithreading
-   Mutual Exclusion
-   Critical Sections
-   Thread Synchronization
-   Condition Variables
-   Deadlock Prevention
-   Starvation Prevention
-   Fair Resource Allocation

------------------------------------------------------------------------

## ⚙️ Configuration

The current implementation uses:

``` text
Philosophers : 4
Chopsticks   : 4
Meals        : 5 per philosopher
Think Time   : 100 ms
Eat Time     : 150 ms
```

These values can be modified in the source code to experiment with
different workloads.

------------------------------------------------------------------------

## 🔄 Execution Flow

``` text
        THINKING
            ↓
          HUNGRY
            ↓
      Join FIFO Queue
            ↓
    ┌─────────────────┐
    │ Can philosopher │
    │      eat?       │
    └───────┬─────────┘
        No  │  Yes
            ↓
          EATING
            ↓
       Release State
            ↓
        THINKING
```

Each philosopher runs in its own thread and repeatedly goes through this
cycle until all required meals are completed.

------------------------------------------------------------------------

## 🛡️ Deadlock Prevention

A traditional solution can result in a situation where every philosopher
holds one chopstick while waiting for the other:

``` text
P0 → waits for P1
P1 → waits for P2
P2 → waits for P3
P3 → waits for P0
```

This creates a **circular wait**.

The waiter-based approach avoids this by controlling when philosophers
are allowed to enter the eating state instead of allowing them to
independently acquire resources in an unsafe order.

------------------------------------------------------------------------

## ⚖️ Fairness

The program also records statistics for each philosopher, including:

-   Number of meals completed
-   Average waiting time
-   Maximum waiting time

A fairness value is calculated using:

``` text
Fairness = Minimum Meals / Maximum Meals
```

A value close to **1.0** indicates that philosophers received a similar
number of opportunities to eat.

------------------------------------------------------------------------

## 📊 Sample Output

``` text
Philosopher 1 is thinking
Philosopher 2 is thinking
Philosopher 3 is thinking
Philosopher 0 is thinking
Philosopher 2 is hungry
Philosopher 2 picked up chopstick 2
Philosopher 2 picked up chopstick 3
Philosopher 2 is EATING (meal 1/5)
Philosopher 1 is hungry
Philosopher 0 is hungry
Philosopher 3 is hungry
Philosopher 2 finished, putting down chopsticks 2 and 3
Philosopher 2 is thinking
Philosopher 1 picked up chopstick 1
Philosopher 1 picked up chopstick 2
Philosopher 1 is EATING (meal 1/5)
Philosopher 2 is hungry
Philosopher 1 finished, putting down chopsticks 1 and 2
Philosopher 1 is thinking
Philosopher 0 picked up chopstick 0
Philosopher 0 picked up chopstick 1
Philosopher 0 is EATING (meal 1/5)
Philosopher 1 is hungry
Philosopher 0 finished, putting down chopsticks 0 and 1
Philosopher 0 is thinking
Philosopher 3 picked up chopstick 3
Philosopher 3 picked up chopstick 0
Philosopher 3 is EATING (meal 1/5)
Philosopher 0 is hungry
Philosopher 3 finished, putting down chopsticks 3 and 0
Philosopher 3 is thinking
Philosopher 2 picked up chopstick 2
Philosopher 2 picked up chopstick 3
Philosopher 2 is EATING (meal 2/5)
Philosopher 3 is hungry
Philosopher 2 finished, putting down chopsticks 2 and 3
Philosopher 2 is thinking
Philosopher 1 picked up chopstick 1
Philosopher 1 picked up chopstick 2
Philosopher 1 is EATING (meal 2/5)
Philosopher 2 is hungry
Philosopher 1 finished, putting down chopsticks 1 and 2
Philosopher 1 is thinking
Philosopher 0 picked up chopstick 0
Philosopher 0 picked up chopstick 1
Philosopher 0 is EATING (meal 2/5)
Philosopher 1 is hungry
Philosopher 0 finished, putting down chopsticks 0 and 1
Philosopher 0 is thinking
Philosopher 3 picked up chopstick 3
Philosopher 3 picked up chopstick 0
Philosopher 3 is EATING (meal 2/5)
Philosopher 0 is hungry
Philosopher 3 finished, putting down chopsticks 3 and 0
Philosopher 3 is thinking
Philosopher 2 picked up chopstick 2
Philosopher 2 picked up chopstick 3
Philosopher 2 is EATING (meal 3/5)
Philosopher 3 is hungry
Philosopher 2 finished, putting down chopsticks 2 and 3
Philosopher 2 is thinking
Philosopher 1 picked up chopstick 1
Philosopher 1 picked up chopstick 2
Philosopher 1 is EATING (meal 3/5)
Philosopher 2 is hungry
Philosopher 1 finished, putting down chopsticks 1 and 2
Philosopher 1 is thinking
Philosopher 0 picked up chopstick 0
Philosopher 0 picked up chopstick 1
Philosopher 0 is EATING (meal 3/5)
Philosopher 1 is hungry
Philosopher 0 finished, putting down chopsticks 0 and 1
Philosopher 3 picked up chopstick 3
Philosopher 3 picked up chopstick 0
Philosopher 3 is EATING (meal 3/5)
Philosopher 0 is thinking
Philosopher 0 is hungry
Philosopher 3 finished, putting down chopsticks 3 and 0
Philosopher 3 is thinking
Philosopher 2 picked up chopstick 2
Philosopher 2 picked up chopstick 3
Philosopher 2 is EATING (meal 4/5)
Philosopher 3 is hungry
Philosopher 2 finished, putting down chopsticks 2 and 3
Philosopher 2 is thinking
Philosopher 1 picked up chopstick 1
Philosopher 1 picked up chopstick 2
Philosopher 1 is EATING (meal 4/5)
Philosopher 2 is hungry
Philosopher 1 finished, putting down chopsticks 1 and 2
Philosopher 1 is thinking
Philosopher 0 picked up chopstick 0
Philosopher 0 picked up chopstick 1
Philosopher 0 is EATING (meal 4/5)
Philosopher 1 is hungry
Philosopher 0 finished, putting down chopsticks 0 and 1
Philosopher 0 is thinking
Philosopher 3 picked up chopstick 3
Philosopher 3 picked up chopstick 0
Philosopher 3 is EATING (meal 4/5)
Philosopher 0 is hungry
Philosopher 3 finished, putting down chopsticks 3 and 0
Philosopher 3 is thinking
Philosopher 2 picked up chopstick 2
Philosopher 2 picked up chopstick 3
Philosopher 2 is EATING (meal 5/5)
Philosopher 3 is hungry
Philosopher 2 finished, putting down chopsticks 2 and 3
Philosopher 2 is done
Philosopher 1 picked up chopstick 1
Philosopher 1 picked up chopstick 2
Philosopher 1 is EATING (meal 5/5)
Philosopher 1 finished, putting down chopsticks 1 and 2
Philosopher 1 is done
Philosopher 0 picked up chopstick 0
Philosopher 0 picked up chopstick 1
Philosopher 0 is EATING (meal 5/5)
Philosopher 0 finished, putting down chopsticks 0 and 1
Philosopher 0 is done
Philosopher 3 picked up chopstick 3
Philosopher 3 picked up chopstick 0
Philosopher 3 is EATING (meal 5/5)
Philosopher 3 finished, putting down chopsticks 3 and 0
Philosopher 3 is done

All philosophers have finished dining.

=== FIFO-fair waiter ===
phil      meals    avg wait (ms)    max wait (ms)
0             5           349.10           361.84
1             5           318.80           364.69
2             5           287.57           360.28
3             5           378.66           459.50
fairness (min/max meals): 1.000  (1.0 = perfectly even)
```

Exact waiting times may vary between executions because thread
scheduling is handled by the operating system.

------------------------------------------------------------------------

## 🚀 How to Run

### Prerequisites

Install [Rust](https://www.rust-lang.org/).

Verify the installation:

``` bash
rustc --version
cargo --version
```

### Clone the Repository

``` bash
git clone https://github.com/nidhishsheka/os_task2.p1.git
cd os_task2.p1
```

### Compile and Run

``` bash
rustc main.rs
./main
```

Or, if using Cargo:

``` bash
cargo run
```

------------------------------------------------------------------------

## 📁 Project Structure

``` text
os_task2.p1/
│
├── main.rs
├── README.md
└── .gitattributes
```

The complete implementation is contained in `main.rs`.

------------------------------------------------------------------------

## 🎓 Learning Outcome

This project provides practical understanding of how Operating Systems
handle **concurrent execution and shared resources**.

It demonstrates how synchronization primitives such as **Mutexes and
Condition Variables** can be combined with a **FIFO scheduling
strategy** to create a safer and fairer concurrent system.

------------------------------------------------------------------------

## 👨‍💻 Author

**H. Nidhish Sheka**\
B.Tech --- Information Science Engineering

[GitHub](https://github.com/nidhishsheka)

------------------------------------------------------------------------

*Operating Systems Assignment --- Dining Philosophers Problem*
