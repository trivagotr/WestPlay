package app.westplay.phone;

/** Never reconstruct private content from lock-screen-redacted notifications. */
final class NotificationPrivacy {
  enum Decision {
    SKIP,
    REDACT,
    FULL
  }

  static Decision decide(boolean secret, boolean locked, boolean publicVisibility) {
    return secret ? Decision.SKIP : locked && !publicVisibility ? Decision.REDACT : Decision.FULL;
  }

  static String title(Decision d, String title) {
    return d == Decision.REDACT ? "New notification" : title;
  }

  static boolean bodyAllowed(Decision d, boolean option) {
    return d == Decision.FULL && option;
  }
}
