package app.westplay.phone;

import java.util.LinkedHashMap;

/**
 * Tracks only transmitted visible content plus the local platform revision; never hidden bodies.
 */
final class NotificationRevisions {
  private record Revision(long postTime, String visible) {}

  private final int capacity;
  private final LinkedHashMap<String, Revision> sent = new LinkedHashMap<>();

  NotificationRevisions(int capacity) {
    if (capacity < 1) throw new IllegalArgumentException();
    this.capacity = capacity;
  }

  synchronized String event(String id, long postTime, String visible) {
    Revision previous = sent.get(id);
    if (previous != null && previous.postTime == postTime && previous.visible.equals(visible))
      return null;
    String event =
        previous != null && previous.postTime == postTime
            ? "notificationUpdated"
            : "notificationPosted";
    if (!sent.containsKey(id) && sent.size() >= capacity)
      sent.remove(sent.keySet().iterator().next());
    sent.put(id, new Revision(postTime, visible));
    return event;
  }

  synchronized boolean remove(String id) {
    return sent.remove(id) != null;
  }

  synchronized void clear() {
    sent.clear();
  }
}
