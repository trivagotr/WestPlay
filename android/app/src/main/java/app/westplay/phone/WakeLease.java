package app.westplay.phone;

/** Finite renewable CPU lease; timeout also bounds a stalled process that cannot run cleanup. */
final class WakeLease {
  interface Lock {
    void acquire(long timeoutMs);

    boolean held();

    void release();
  }

  interface Scheduler {
    void later(Runnable task, long delayMs);

    void cancel(Runnable task);
  }

  private final Lock lock;
  private final Scheduler scheduler;
  private boolean active;
  private long generation;
  private Runnable renewal;

  WakeLease(Lock lock, Scheduler scheduler) {
    this.lock = lock;
    this.scheduler = scheduler;
  }

  void start() {
    if (active) return;
    active = true;
    long current = ++generation;
    renewal =
        new Runnable() {
          public void run() {
            if (!active || generation != current) return;
            try {
              lock.acquire(120000);
              scheduler.later(this, 60000);
            } catch (RuntimeException e) {
              stop();
            }
          }
        };
    renewal.run();
  }

  void stop() {
    active = false;
    generation++;
    if (renewal != null) {
      scheduler.cancel(renewal);
      renewal = null;
    }
    if (lock.held()) lock.release();
  }
}
