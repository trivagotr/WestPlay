package app.westplay.phone;

import android.content.*;
import android.security.keystore.*;
import android.util.Base64;
import java.security.*;
import java.util.UUID;
import javax.crypto.*;
import javax.crypto.spec.GCMParameterSpec;
import org.json.*;

final class CredentialStore {
  private final android.content.SharedPreferences prefs;
  private static final String ALIAS = "westplay-phone-v1";

  CredentialStore(Context c) {
    prefs = c.getSharedPreferences("connection", Context.MODE_PRIVATE);
  }

  private javax.crypto.SecretKey key() throws Exception {
    KeyStore k = KeyStore.getInstance("AndroidKeyStore");
    k.load(null);
    if (!k.containsAlias(ALIAS)) {
      KeyGenerator g = KeyGenerator.getInstance("AES", "AndroidKeyStore");
      g.init(
          new KeyGenParameterSpec.Builder(
                  ALIAS, KeyProperties.PURPOSE_ENCRYPT | KeyProperties.PURPOSE_DECRYPT)
              .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
              .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
              .build());
      g.generateKey();
    }
    return (javax.crypto.SecretKey) k.getKey(ALIAS, null);
  }

  synchronized JSONObject read() {
    try {
      String s = prefs.getString("sealed", null);
      if (s == null) return null;
      String[] a = s.split(":");
      Cipher c = Cipher.getInstance("AES/GCM/NoPadding");
      c.init(Cipher.DECRYPT_MODE, key(), new GCMParameterSpec(128, Base64.decode(a[0], 0)));
      return new JSONObject(
          new String(c.doFinal(Base64.decode(a[1], 0)), java.nio.charset.StandardCharsets.UTF_8));
    } catch (Exception e) {
      return null;
    }
  }

  synchronized void save(JSONObject data) {
    try {
      Cipher c = Cipher.getInstance("AES/GCM/NoPadding");
      c.init(Cipher.ENCRYPT_MODE, key());
      String s =
          Base64.encodeToString(c.getIV(), 2)
              + ":"
              + Base64.encodeToString(
                  c.doFinal(data.toString().getBytes(java.nio.charset.StandardCharsets.UTF_8)), 2);
      if (!prefs.edit().putString("sealed", s).commit()) throw new IllegalStateException();
    } catch (Exception e) {
      throw new IllegalStateException("Secure storage unavailable", e);
    }
  }

  synchronized void forget() {
    prefs.edit().remove("sealed").commit();
  }

  String deviceId() {
    String s = prefs.getString("deviceId", null);
    if (s == null) {
      s = UUID.randomUUID().toString();
      prefs.edit().putString("deviceId", s).commit();
    }
    return s;
  }

  android.content.SharedPreferences options() {
    return prefs;
  }

  static PhoneProtocol.Endpoint endpoint(JSONObject j) {
    return new PhoneProtocol.Endpoint(
        j.optString("pc"), j.optString("host"), j.optInt("port"), j.optString("fp"), null);
  }

  static JSONObject record(PhoneProtocol.Endpoint e, String token) {
    try {
      return new JSONObject()
          .put("pc", e.pcId())
          .put("host", e.host())
          .put("port", e.port())
          .put("fp", e.pin())
          .put("token", token);
    } catch (Exception x) {
      throw new IllegalStateException(x);
    }
  }
}
