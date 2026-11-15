package app.westplay.phone;

import static org.junit.Assert.*;

import org.json.JSONObject;
import org.junit.Test;

public class PhoneProtocolTest {
  String fp = "a".repeat(64), invite = "b".repeat(64);

  String uri(String pc, String host, String port, String pin) {
    return "westplay://phone?v=1&pc="
        + pc
        + "&host="
        + host
        + "&port="
        + port
        + "&fp="
        + pin
        + "&invite="
        + invite;
  }

  @Test
  public void parsesPairAndRedactsSecrets() {
    var e = PhoneProtocol.parse(uri("pc1", "192.168.1.2", "42000", fp));
    assertEquals(42000, e.port());
    assertFalse(e.toString().contains(invite));
  }

  @Test
  public void rejectsUnsafeEndpoints() {
    for (String h : new String[] {"8.8.8.8", "127.0.0.1", "192.168.1.999", "evil.example"})
      assertThrows(
          IllegalArgumentException.class, () -> PhoneProtocol.parse(uri("pc", "" + h, "4000", fp)));
    assertThrows(
        IllegalArgumentException.class,
        () -> PhoneProtocol.parse(uri("pc", "192.168.1.2", "0", fp)));
    assertThrows(
        IllegalArgumentException.class,
        () -> PhoneProtocol.parse(uri("pc", "192.168.1.2", "2", "short")));
    assertThrows(
        IllegalArgumentException.class,
        () -> PhoneProtocol.parse(uri("pc", "192.168.1.2", "2", fp) + "&pc=other"));
  }

  @Test
  public void endpointChangesRequireSameIdentityAndPin() {
    var a = PhoneProtocol.parse(uri("pc", "192.168.1.2", "4000", fp));
    var next = PhoneProtocol.parse(uri("pc", "10.0.0.2", "4001", fp));
    assertEquals("10.0.0.2", PhoneProtocol.refresh(a, next).host());
    assertThrows(
        IllegalArgumentException.class,
        () -> PhoneProtocol.refresh(a, PhoneProtocol.parse(uri("other", "10.0.0.2", "4001", fp))));
    assertThrows(
        IllegalArgumentException.class,
        () ->
            PhoneProtocol.refresh(
                a, PhoneProtocol.parse(uri("pc", "10.0.0.2", "4001", "c".repeat(64)))));
  }

  @Test
  public void unicodeBoundsAndTypedEnvelope() throws Exception {
    assertEquals("😀😀", PhoneProtocol.clip("😀😀😀", 2));
    String s = PhoneProtocol.event(1, "notificationPosted", new JSONObject().put("title", "x"));
    var j = new JSONObject(s);
    assertEquals(1, j.getInt("version"));
    assertEquals(1, j.getInt("sequence"));
    assertEquals("notificationPosted", j.getString("eventType"));
    assertThrows(
        IllegalArgumentException.class,
        () -> PhoneProtocol.event(0, "heartbeat", new JSONObject()));
    assertThrows(
        IllegalArgumentException.class, () -> PhoneProtocol.event(1, "bogus", new JSONObject()));
    assertThrows(
        IllegalArgumentException.class,
        () ->
            PhoneProtocol.event(
                1, "notificationPosted", new JSONObject().put("body", "x".repeat(9000))));
  }

  @Test
  public void rejectsOversizedNotificationFields() {
    assertThrows(
        IllegalArgumentException.class,
        () ->
            PhoneProtocol.event(
                1, "notificationPosted", new JSONObject().put("title", "x".repeat(161))));
    assertThrows(
        IllegalArgumentException.class,
        () ->
            PhoneProtocol.event(
                1, "notificationPosted", new JSONObject().put("body", "😀".repeat(501))));
  }

  @Test
  public void endpointUpdateWithoutInviteIsValid() {
    var e = PhoneProtocol.parse(uri("pc1", "192.168.1.2", "42000", fp).split("&invite=")[0]);
    assertNull(e.invite());
  }
}
