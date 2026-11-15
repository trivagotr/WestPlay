package app.westplay.phone;

import static org.junit.Assert.*;

import org.junit.Test;

public class NotificationRevisionsTest {
  @Test
  public void freshPostTimeIsPostedButSameRevisionContentChangeIsUpdated() {
    var r = new NotificationRevisions(4);
    assertEquals("notificationPosted", r.event("conversation", 10, "visible"));
    assertNull(r.event("conversation", 10, "visible"));
    assertEquals("notificationUpdated", r.event("conversation", 10, "changed visible"));
    assertEquals("notificationPosted", r.event("conversation", 11, "changed visible"));
  }

  @Test
  public void bodyOffIdenticalVisibleTextStillAllowsNewArrival() {
    var r = new NotificationRevisions(4);
    assertEquals("notificationPosted", r.event("contact", 1, "title;body=null"));
    assertEquals("notificationPosted", r.event("contact", 2, "title;body=null"));
    assertTrue(r.remove("contact"));
    assertFalse(r.remove("contact"));
    assertEquals("notificationPosted", r.event("contact", 2, "title;body=null"));
    r.clear();
    assertEquals("notificationPosted", r.event("contact", 2, "title;body=null"));
  }
}
