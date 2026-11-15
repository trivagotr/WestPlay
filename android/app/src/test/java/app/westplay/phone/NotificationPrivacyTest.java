package app.westplay.phone;

import static org.junit.Assert.*;

import org.junit.Test;

public class NotificationPrivacyTest {
  @Test
  public void secretNeverLeavesPhone() {
    assertEquals(NotificationPrivacy.Decision.SKIP, NotificationPrivacy.decide(true, false, true));
    assertEquals(NotificationPrivacy.Decision.SKIP, NotificationPrivacy.decide(true, true, false));
  }

  @Test
  public void privateLockedArrivalUsesGenericTitleWithoutBody() {
    var d = NotificationPrivacy.decide(false, true, false);
    assertEquals(NotificationPrivacy.Decision.REDACT, d);
    assertEquals("New notification", NotificationPrivacy.title(d, "Secret contact text"));
    assertFalse(NotificationPrivacy.bodyAllowed(d, true));
  }

  @Test
  public void publicAndUnlockedFollowExplicitBodyChoice() {
    assertEquals(NotificationPrivacy.Decision.FULL, NotificationPrivacy.decide(false, true, true));
    var d = NotificationPrivacy.decide(false, false, false);
    assertEquals("Visible title", NotificationPrivacy.title(d, "Visible title"));
    assertFalse(NotificationPrivacy.bodyAllowed(d, false));
    assertTrue(NotificationPrivacy.bodyAllowed(d, true));
  }
}
