package app.westplay.phone;

import static org.junit.Assert.*;

import org.junit.Test;

public class WakeLeaseTest {
  static class Lock implements WakeLease.Lock {
    int acquired, released;
    long timeout;
    boolean held;

    public void acquire(long t) {
      acquired++;
      timeout = t;
      held = true;
    }

    public boolean held() {
      return held;
    }

    public void release() {
      released++;
      held = false;
    }
  }

  static class Tasks implements WakeLease.Scheduler {
    Runnable task;
    long delay;
    int canceled;

    public void later(Runnable r, long d) {
      task = r;
      delay = d;
    }

    public void cancel(Runnable r) {
      if (task == r) task = null;
      canceled++;
    }
  }

  @Test
  public void activeLeaseHasFiniteTimeoutAndRenewsBeforeExpiration() {
    var l = new Lock();
    var t = new Tasks();
    var lease = new WakeLease(l, t);
    lease.start();
    assertEquals(1, l.acquired);
    assertEquals(120000, l.timeout);
    assertEquals(60000, t.delay);
    t.task.run();
    assertEquals(2, l.acquired);
    assertNotNull(t.task);
    lease.start();
    assertEquals(2, l.acquired);
  }

  @Test
  public void stopReleasesAndStaleRenewalCannotReacquire() {
    var l = new Lock();
    var t = new Tasks();
    var lease = new WakeLease(l, t);
    lease.start();
    Runnable stale = t.task;
    lease.stop();
    assertEquals(1, l.released);
    assertNull(t.task);
    stale.run();
    assertEquals(1, l.acquired);
    lease.stop();
    assertEquals(1, l.released);
    lease.start();
    assertEquals(2, l.acquired);
    stale.run();
    assertEquals(2, l.acquired);
  }
}
