package app.westplay.phone;

import java.net.URI;
import java.net.URLDecoder;
import java.nio.charset.StandardCharsets;
import java.util.*;
import org.json.JSONObject;

public final class PhoneProtocol {
  public record Endpoint(String pcId, String host, int port, String pin, String invite) {
    @Override
    public String toString() {
      return "PC " + pcId + " at " + host + ":" + port;
    }
  }

  public static Endpoint parse(String value) {
    try {
      URI u = URI.create(value.trim());
      if (!"westplay".equals(u.getScheme())
          || !"phone".equals(u.getHost())
          || u.getFragment() != null
          || u.getUserInfo() != null
          || u.getPort() != -1
          || !(u.getPath() == null || u.getPath().isEmpty()))
        throw new IllegalArgumentException("Invalid pairing link");
      Map<String, String> q = new HashMap<>();
      for (String part : u.getRawQuery().split("&")) {
        String[] v = part.split("=", 2);
        if (v.length != 2) throw new IllegalArgumentException();
        String k = URLDecoder.decode(v[0], "UTF-8"), s = URLDecoder.decode(v[1], "UTF-8");
        if (q.put(k, s) != null) throw new IllegalArgumentException("Duplicate field");
      }
      if (!"1".equals(q.get("v"))) throw new IllegalArgumentException("Unsupported protocol");
      String pc = q.get("pc"), host = q.get("host"), pin = q.get("fp"), invite = q.get("invite");
      int port = Integer.parseInt(q.get("port"));
      if (pc == null
          || !pc.matches("[A-Za-z0-9_-]{1,160}")
          || !privateIp(host)
          || port < 1
          || port > 65535
          || pin == null
          || !pin.matches("[0-9a-f]{64}")
          || (invite != null && !invite.matches("[0-9a-f]{64}")))
        throw new IllegalArgumentException("Invalid PC endpoint");
      return new Endpoint(pc, host, port, pin, invite);
    } catch (Exception e) {
      throw new IllegalArgumentException("Invalid WestPlay pairing link");
    }
  }

  public static boolean privateIp(String s) {
    try {
      String[] p = s.split("\\.", -1);
      if (p.length != 4) return false;
      int[] a = new int[4];
      for (int i = 0; i < 4; i++) {
        if (!p[i].matches("0|[1-9][0-9]{0,2}")) return false;
        a[i] = Integer.parseInt(p[i]);
        if (a[i] > 255) return false;
      }
      return a[0] == 10
          || (a[0] == 172 && a[1] >= 16 && a[1] <= 31)
          || (a[0] == 192 && a[1] == 168);
    } catch (Exception e) {
      return false;
    }
  }

  public static Endpoint refresh(Endpoint saved, Endpoint found) {
    if (!saved.pcId.equals(found.pcId) || !saved.pin.equals(found.pin))
      throw new IllegalArgumentException(
          "PC identity or key changed; forget before pairing a new PC");
    return new Endpoint(saved.pcId, found.host, found.port, saved.pin, null);
  }

  public static String clip(String s, int n) {
    if (s == null) return "";
    int count = s.codePointCount(0, s.length());
    return count <= n ? s : s.substring(0, s.offsetByCodePoints(0, n));
  }

  public static String event(long n, String kind, JSONObject payload) {
    try {
      if (n < 1
          || !Arrays.asList(
                  "deviceHello",
                  "deviceState",
                  "notificationPosted",
                  "notificationUpdated",
                  "notificationRemoved",
                  "heartbeat",
                  "sessionEnded")
              .contains(kind)) throw new IllegalArgumentException("Invalid event");
      if (kind.startsWith("notification")) {
        String[] names = {"id", "appId", "appName", "title", "body"};
        int[] limits = {160, 255, 80, 160, 500};
        for (int i = 0; i < names.length; i++) {
          Object v = payload.opt(names[i]);
          if (v != null
              && v != JSONObject.NULL
              && (!(v instanceof String)
                  || ((String) v).codePointCount(0, ((String) v).length()) > limits[i]))
            throw new IllegalArgumentException("Notification field exceeds limit");
        }
      }
      String s =
          new JSONObject()
              .put("version", 1)
              .put("sequence", n)
              .put("eventType", kind)
              .put("payload", payload)
              .toString();
      if (s.getBytes(StandardCharsets.UTF_8).length > 8192)
        throw new IllegalArgumentException("Event too large");
      return s;
    } catch (org.json.JSONException e) {
      throw new IllegalArgumentException(e);
    }
  }

  public static String sha256(byte[] bytes) {
    try {
      byte[] h = java.security.MessageDigest.getInstance("SHA-256").digest(bytes);
      StringBuilder b = new StringBuilder();
      for (byte x : h) b.append(String.format(Locale.ROOT, "%02x", x & 255));
      return b.toString();
    } catch (Exception e) {
      throw new IllegalStateException(e);
    }
  }
}
