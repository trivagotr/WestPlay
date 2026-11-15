package app.westplay.phone;

import java.util.*;
import java.util.function.Consumer;

/** Serial resolver; an uncancellable old request keeps its slot until its callback completes. */
final class ResolutionQueue<T> {
  interface Resolver<T> {
    void resolve(T item, Consumer<T> completed);
  }

  private final int max;
  private final Resolver<T> resolver;
  private final LinkedHashMap<String, T> pending = new LinkedHashMap<>();
  private long generation;
  private Consumer<T> found;
  private Ticket active;

  private final class Ticket {
    final long generation;
    final String key;
    boolean removed;

    Ticket(long g, String k) {
      generation = g;
      key = k;
    }
  }

  ResolutionQueue(int max, Resolver<T> resolver) {
    if (max < 1) throw new IllegalArgumentException();
    this.max = max;
    this.resolver = resolver;
  }

  long start(Consumer<T> found) {
    stop();
    this.found = found;
    return generation;
  }

  void stop() {
    generation++;
    found = null;
    pending.clear();
  }

  void offer(long g, String key, T item) {
    if (g != generation || found == null) return;
    if (active != null && active.generation == g && active.key.equals(key)) return;
    if (!pending.containsKey(key) && pending.size() >= max)
      pending.remove(pending.keySet().iterator().next());
    pending.put(key, item);
    drain();
  }

  void remove(long g, String key) {
    if (g != generation) return;
    pending.remove(key);
    if (active != null && active.generation == g && active.key.equals(key)) active.removed = true;
  }

  private void drain() {
    if (active != null || found == null || pending.isEmpty()) return;
    var entry = pending.entrySet().iterator().next();
    String key = entry.getKey();
    T item = entry.getValue();
    pending.remove(key);
    Ticket ticket = new Ticket(generation, key);
    active = ticket;
    try {
      resolver.resolve(item, result -> complete(ticket, result));
    } catch (Exception e) {
      complete(ticket, null);
    }
  }

  private void complete(Ticket ticket, T result) {
    if (active != ticket) return;
    active = null;
    try {
      if (ticket.generation == generation && !ticket.removed && found != null && result != null)
        found.accept(result);
    } finally {
      drain();
    }
  }
}
