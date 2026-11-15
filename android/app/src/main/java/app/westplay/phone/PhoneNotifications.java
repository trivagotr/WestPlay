package app.westplay.phone;

import android.app.*;
import android.content.*;
import android.service.notification.*;
import java.util.*;
import org.json.*;

public final class PhoneNotifications extends NotificationListenerService {
  static boolean access(Context c) {
    String enabled =
        android.provider.Settings.Secure.getString(
            c.getContentResolver(), "enabled_notification_listeners");
    if (enabled == null) return false;
    ComponentName own = new ComponentName(c, PhoneNotifications.class);
    for (String name : enabled.split(":"))
      if (own.equals(ComponentName.unflattenFromString(name))) return true;
    return false;
  }

  private static final NotificationRevisions sent = new NotificationRevisions(256);

  static synchronized void clearTracked() {
    sent.clear();
  }

  public void onListenerConnected() {
    RelayService s = RelayService.current;
    if (s != null) s.optionsChanged();
  }

  public void onListenerDisconnected() {
    clearTracked();
    RelayService s = RelayService.current;
    if (s != null) s.optionsChanged();
  }

  public void onNotificationPosted(StatusBarNotification sbn) {
    RelayService s = RelayService.current;
    if (s == null || !s.connected()) return;
    Notification n = sbn.getNotification();
    String pkg = sbn.getPackageName();
    var prefs = new CredentialStore(this).options();
    Set<String> blocked =
        new HashSet<>(Arrays.asList(prefs.getString("blocked", "").split("[,\\s]+")));
    String id =
        PhoneProtocol.sha256(sbn.getKey().getBytes(java.nio.charset.StandardCharsets.UTF_8));
    boolean locked = ((KeyguardManager) getSystemService(KEYGUARD_SERVICE)).isDeviceLocked();
    var privacy =
        NotificationPrivacy.decide(
            n.visibility == Notification.VISIBILITY_SECRET,
            locked,
            n.visibility == Notification.VISIBILITY_PUBLIC);
    if (pkg.equals(getPackageName())
        || blocked.contains(pkg)
        || (n.flags & (Notification.FLAG_ONGOING_EVENT | Notification.FLAG_FOREGROUND_SERVICE)) != 0
        || privacy == NotificationPrivacy.Decision.SKIP
        || Notification.CATEGORY_TRANSPORT.equals(n.category)
        || (n.extras.getInt(Notification.EXTRA_PROGRESS_MAX, 0) > 0
            || n.extras.getBoolean(Notification.EXTRA_PROGRESS_INDETERMINATE, false))) {
      removeSent(s, id);
      return;
    }
    try {
      CharSequence
          title =
              privacy == NotificationPrivacy.Decision.FULL
                  ? n.extras.getCharSequence(Notification.EXTRA_TITLE)
                  : null,
          body =
              privacy == NotificationPrivacy.Decision.FULL
                  ? n.extras.getCharSequence(Notification.EXTRA_TEXT)
                  : null;
      String app = pkg;
      try {
        app =
            getPackageManager()
                .getApplicationLabel(getPackageManager().getApplicationInfo(pkg, 0))
                .toString();
      } catch (Exception ignored) {
      }
      JSONObject data =
          new JSONObject()
              .put("id", id)
              .put("appId", PhoneProtocol.clip(pkg, 255))
              .put("appName", PhoneProtocol.clip(app, 80))
              .put(
                  "title",
                  PhoneProtocol.clip(
                      NotificationPrivacy.title(privacy, title == null ? "" : title.toString()),
                      160))
              .put(
                  "body",
                  NotificationPrivacy.bodyAllowed(privacy, prefs.getBoolean("body", false))
                      ? PhoneProtocol.clip(body == null ? "" : body.toString(), 500)
                      : JSONObject.NULL);
      String event = sent.event(id, sbn.getPostTime(), data.toString());
      if (event != null) s.emit(event, data);
    } catch (Exception ignored) {
    }
  }

  private static void removeSent(RelayService s, String id) {
    boolean wasSent;
    synchronized (PhoneNotifications.class) {
      wasSent = sent.remove(id);
    }
    if (wasSent && s != null)
      try {
        s.emit("notificationRemoved", new JSONObject().put("id", id));
      } catch (Exception ignored) {
      }
  }

  public void onNotificationRemoved(StatusBarNotification sbn) {
    removeSent(
        RelayService.current,
        PhoneProtocol.sha256(sbn.getKey().getBytes(java.nio.charset.StandardCharsets.UTF_8)));
  }
}
