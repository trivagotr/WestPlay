package app.westplay.phone;

import static org.junit.Assert.*;

import java.util.*;
import java.util.function.Consumer;
import org.junit.Test;

public class ResolutionQueueTest {
  @Test
  public void resolvesTargetDiscoveredWhileAnotherPcIsResolving() {
    List<String> started = new ArrayList<>(), found = new ArrayList<>();
    List<Consumer<String>> callbacks = new ArrayList<>();
    var q =
        new ResolutionQueue<String>(
            8,
            (item, done) -> {
              started.add(item);
              callbacks.add(done);
            });
    long gen = q.start(found::add);
    q.offer(gen, "other", "other");
    q.offer(gen, "target", "target");
    assertEquals(Arrays.asList("other"), started);
    callbacks.get(0).accept("other resolved");
    assertEquals(Arrays.asList("other", "target"), started);
    callbacks.get(1).accept("target resolved");
    assertTrue(found.contains("target resolved"));
  }

  @Test
  public void staleCompletionCannotPublishOrReleaseNewSessionResolve() {
    List<String> started = new ArrayList<>(), found = new ArrayList<>();
    List<Consumer<String>> callbacks = new ArrayList<>();
    var q =
        new ResolutionQueue<String>(
            8,
            (item, done) -> {
              started.add(item);
              callbacks.add(done);
            });
    long old = q.start(found::add);
    q.offer(old, "old", "old");
    q.stop();
    long current = q.start(found::add);
    q.offer(old, "stale", "stale");
    q.offer(current, "new", "new");
    callbacks.get(0).accept("old resolved");
    assertTrue(found.isEmpty());
    assertEquals(Arrays.asList("old", "new"), started);
    callbacks.get(0).accept("duplicate stale completion");
    q.offer(current, "next", "next");
    assertEquals(2, started.size());
    callbacks.get(1).accept("new resolved");
    assertEquals(Arrays.asList("new resolved"), found);
    assertEquals(Arrays.asList("old", "new", "next"), started);
  }

  @Test
  public void pendingQueueIsBoundedAndLostServicesAreRemoved() {
    List<String> started = new ArrayList<>();
    List<Consumer<String>> callbacks = new ArrayList<>();
    var q =
        new ResolutionQueue<String>(
            2,
            (item, done) -> {
              started.add(item);
              callbacks.add(done);
            });
    long gen = q.start(x -> {});
    q.offer(gen, "active", "active");
    q.offer(gen, "a", "a");
    q.offer(gen, "b", "b");
    q.offer(gen, "c", "c");
    q.remove(gen, "b");
    callbacks.get(0).accept(null);
    assertEquals(Arrays.asList("active", "c"), started);
  }
}
