package app.westplay.phone;

import android.app.*;
import android.content.*;
import android.net.wifi.WifiManager;
import android.os.*;
import java.security.cert.*;
import java.util.concurrent.TimeUnit;
import javax.net.ssl.*;
import okhttp3.*;
import org.json.*;

public final class RelayService extends Service {
  static volatile RelayService current;
  static volatile String status = "Stopped";
  private final Handler h = new Handler(Looper.getMainLooper());
  private CredentialStore store;
  private WakeLease wakeLease;
  private Discovery discovery;
  private PhoneProtocol.Endpoint endpoint;
  private String token, invite;
  private OkHttpClient client;
  private WebSocket ws;
  private volatile boolean ready;
  private boolean running;
  private long sequence, lastReceive;
  private int attempt;
  private final java.util.concurrent.atomic.AtomicInteger pending =
      new java.util.concurrent.atomic.AtomicInteger();
  private WifiManager.MulticastLock multicast;
  private final BroadcastReceiver battery =
      new BroadcastReceiver() {
        public void onReceive(Context c, Intent i) {
          sendBattery(i);
        }
      };

  public IBinder onBind(Intent i) {
    return null;
  }

  public void onCreate() {
    super.onCreate();
    store = new CredentialStore(this);
    PowerManager.WakeLock cpu =
        ((PowerManager) getSystemService(POWER_SERVICE))
            .newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "WestPlay:PhoneRelay");
    cpu.setReferenceCounted(false);
    wakeLease =
        new WakeLease(
            new WakeLease.Lock() {
              public void acquire(long timeoutMs) {
                cpu.acquire(timeoutMs);
              }

              public boolean held() {
                return cpu.isHeld();
              }

              public void release() {
                cpu.release();
              }
            },
            new WakeLease.Scheduler() {
              public void later(Runnable task, long delayMs) {
                h.postDelayed(task, delayMs);
              }

              public void cancel(Runnable task) {
                h.removeCallbacks(task);
              }
            });
    discovery = new Discovery(this);
    current = this;
    ((NotificationManager) getSystemService(NOTIFICATION_SERVICE))
        .createNotificationChannel(
            new NotificationChannel(
                "link", "WestPlay phone link", NotificationManager.IMPORTANCE_LOW));
    registerReceiver(battery, new IntentFilter(Intent.ACTION_BATTERY_CHANGED));
  }

  public int onStartCommand(Intent i, int flags, int id) {
    if (i != null && "stop".equals(i.getAction())) {
      shutdown();
      return START_NOT_STICKY;
    }
    if (running) {
      String update = i == null ? null : i.getStringExtra("uri");
      if (update != null && !update.trim().isEmpty()) {
        try {
          var fresh = PhoneProtocol.parse(update);
          endpoint = PhoneProtocol.refresh(endpoint, fresh);
          if (token != null) store.save(CredentialStore.record(endpoint, token));
          else if (fresh.invite() != null) invite = fresh.invite();
          ready = false;
          if (ws != null) {
            ws.cancel();
            ws = null;
          }
          if (client != null) {
            client.connectionPool().evictAll();
            client.dispatcher().executorService().shutdown();
            client = null;
          }
          PhoneNotifications.clearTracked();
          discover();
          connect();
        } catch (Exception e) {
          state("Endpoint update rejected: PC identity or key differs");
        }
      } else if (ws == null) {
        discover();
        connect();
      }
      return START_NOT_STICKY;
    }
    running = true;
    startForeground(1, notice("Connecting to PC"));
    try {
      wakeLease.start();
      JSONObject saved = store.read();
      String uri = i == null ? null : i.getStringExtra("uri");
      if (uri != null && !uri.trim().isEmpty()) {
        var fresh = PhoneProtocol.parse(uri);
        if (saved != null) {
          endpoint = PhoneProtocol.refresh(CredentialStore.endpoint(saved), fresh);
          token = saved.optString("token", null);
          store.save(CredentialStore.record(endpoint, token));
        } else {
          if (fresh.invite() == null)
            throw new IllegalArgumentException("First pairing needs the full QR link");
          endpoint = fresh;
          invite = fresh.invite();
        }
      } else {
        if (saved == null)
          throw new IllegalArgumentException("Scan or paste your PC pairing link first");
        endpoint = CredentialStore.endpoint(saved);
        token = saved.optString("token", null);
      }
      WifiManager wifi = (WifiManager) getApplicationContext().getSystemService(WIFI_SERVICE);
      if (wifi != null) {
        multicast = wifi.createMulticastLock("westplay-discovery");
        multicast.setReferenceCounted(false);
        multicast.acquire();
      }
      discover();
      connect();
    } catch (Exception e) {
      status = e.getMessage();
      shutdown();
    }
    return START_NOT_STICKY;
  }

  private void discover() {
    discovery.start(
        endpoint.pcId(),
        s ->
            h.post(
                () -> {
                  if (!running || ready || endpoint == null) return;
                  endpoint =
                      new PhoneProtocol.Endpoint(
                          endpoint.pcId(),
                          s.getHost().getHostAddress(),
                          s.getPort(),
                          endpoint.pin(),
                          null);
                  if (token != null) store.save(CredentialStore.record(endpoint, token));
                  if (ws == null) connect();
                }));
  }

  private Notification notice(String text) {
    Intent stop = new Intent(this, RelayService.class).setAction("stop");
    return new Notification.Builder(this, "link")
        .setContentTitle("WestPlay phone link")
        .setContentText(text)
        .setSmallIcon(android.R.drawable.stat_sys_upload)
        .setOngoing(true)
        .setContentIntent(
            PendingIntent.getActivity(
                this, 0, new Intent(this, MainActivity.class), PendingIntent.FLAG_IMMUTABLE))
        .addAction(
            new Notification.Action.Builder(
                    null,
                    "Stop",
                    PendingIntent.getService(this, 1, stop, PendingIntent.FLAG_IMMUTABLE))
                .build())
        .build();
  }

  private void state(String s) {
    status = s;
    ((NotificationManager) getSystemService(NOTIFICATION_SERVICE)).notify(1, notice(s));
  }

  private void connect() {
    if (!running || ws != null) return;
    ready = false;
    sequence = 0;
    PhoneNotifications.clearTracked();
    state("Connecting securely to " + endpoint.host() + ":" + endpoint.port());
    try {
      String pin = endpoint.pin();
      X509TrustManager trust =
          new X509TrustManager() {
            public X509Certificate[] getAcceptedIssuers() {
              return new X509Certificate[0];
            }

            public void checkClientTrusted(X509Certificate[] c, String a)
                throws CertificateException {
              throw new CertificateException();
            }

            public void checkServerTrusted(X509Certificate[] c, String a)
                throws CertificateException {
              if (c == null || c.length == 0) throw new CertificateException("No certificate");
              c[0].checkValidity();
              if (!PhoneProtocol.sha256(c[0].getPublicKey().getEncoded()).equals(pin))
                throw new CertificateException("PC key does not match saved QR");
            }
          };
      SSLContext ssl = SSLContext.getInstance("TLS");
      ssl.init(null, new TrustManager[] {trust}, null);
      client =
          new OkHttpClient.Builder()
              .sslSocketFactory(ssl.getSocketFactory(), trust)
              .connectTimeout(8, TimeUnit.SECONDS)
              .readTimeout(0, TimeUnit.MILLISECONDS)
              .build();
      final WebSocketListener listener =
          new WebSocketListener() {
            public void onOpen(WebSocket socket, Response response) {
              h.post(
                  () -> {
                    if (ws != socket || !running) {
                      socket.cancel();
                      return;
                    }
                    lastReceive = SystemClock.elapsedRealtime();
                    try {
                      JSONObject auth =
                          new JSONObject()
                              .put("version", 1)
                              .put("type", token == null ? "pair" : "authenticate")
                              .put(
                                  token == null ? "invite" : "token",
                                  token == null ? invite : token)
                              .put("deviceId", store.deviceId())
                              .put(
                                  "deviceName",
                                  PhoneProtocol.clip(Build.MANUFACTURER + " " + Build.MODEL, 80))
                              .put("platform", "android");
                      socket.send(auth.toString());
                      h.postDelayed(
                          () -> {
                            if (ws == socket && !ready) lost(socket, "Handshake timeout");
                          },
                          10000);
                    } catch (Exception e) {
                      lost(socket, "Handshake failed");
                    }
                  });
            }

            public void onMessage(WebSocket socket, String text) {
              h.post(() -> receive(socket, text));
            }

            public void onFailure(WebSocket socket, Throwable t, Response r) {
              h.post(() -> lost(socket, "Connection lost; retrying known PC"));
            }

            public void onClosed(WebSocket socket, int code, String reason) {
              h.post(() -> lost(socket, "PC disconnected; retrying"));
            }
          };
      ws =
          client.newWebSocket(
              new Request.Builder()
                  .url("wss://" + endpoint.host() + ":" + endpoint.port() + "/phone")
                  .build(),
              listener);
    } catch (Exception e) {
      retry("Secure connection failed");
    }
  }

  private void receive(WebSocket socket, String text) {
    if (socket != ws || !running) return;
    try {
      if (text.length() > 8192) throw new IllegalArgumentException();
      JSONObject j = new JSONObject(text);
      if (j.getInt("version") != 1) throw new IllegalArgumentException();
      lastReceive = SystemClock.elapsedRealtime();
      String type = j.getString("type");
      if ("error".equals(type)) {
        String code = j.optString("code");
        if ("revoked".equals(code) || "unauthorized".equals(code)) {
          store.forget();
          state("Pairing revoked or unauthorized; pair again");
          shutdown();
        } else if ("serviceStopped".equals(code)
            || "alreadyPaired".equals(code)
            || "expiredInvite".equals(code)
            || "protocolMismatch".equals(code)) {
          state("PC: " + code);
          shutdown();
        } else lost(socket, "PC rejected connection");
        return;
      }
      if (!ready) {
        if (!"ready".equals(type) || !endpoint.pcId().equals(j.getString("pcId")))
          throw new IllegalArgumentException();
        if (token == null) {
          token = j.getString("token");
          if (!token.matches("[0-9a-f]{64}")) throw new IllegalArgumentException();
          store.save(CredentialStore.record(endpoint, token));
          invite = null;
        }
        ready = true;
        attempt = 0;
        state("Connected • battery and enabled notifications");
        optionsChanged();
        sendBattery(registerReceiver(null, new IntentFilter(Intent.ACTION_BATTERY_CHANGED)));
        heartbeat(socket);
      } else if (!"ack".equals(type)) throw new IllegalArgumentException();
    } catch (Exception e) {
      lost(socket, "Invalid PC handshake or response");
    }
  }

  private void heartbeat(WebSocket socket) {
    h.postDelayed(
        () -> {
          if (ws != socket || !ready) return;
          if (SystemClock.elapsedRealtime() - lastReceive > 20000) {
            lost(socket, "PC stopped responding");
            return;
          }
          emit("heartbeat", new JSONObject());
          heartbeat(socket);
        },
        5000);
  }

  private void lost(WebSocket socket, String reason) {
    if (socket != ws) return;
    ws = null;
    socket.cancel();
    ready = false;
    PhoneNotifications.clearTracked();
    if (client != null) {
      client.connectionPool().evictAll();
      client.dispatcher().executorService().shutdown();
      client = null;
    }
    if (running) retry(reason);
  }

  private void retry(String reason) {
    state(reason);
    discover();
    h.postDelayed(
        () -> {
          if (running && ws == null) connect();
        },
        Math.min(30000, 1000L * (1L << Math.min(attempt++, 5))));
  }

  void emit(String type, JSONObject payload) {
    WebSocket session = ws;
    if (!ready || session == null) return;
    if (pending.incrementAndGet() > 64) {
      pending.decrementAndGet();
      return;
    }
    h.post(
        () -> {
          pending.decrementAndGet();
          if (!ready || ws != session) return;
          try {
            if (ws.queueSize() > 32768) {
              lost(ws, "Link queue full; retrying");
              return;
            }
            if (!ws.send(PhoneProtocol.event(++sequence, type, payload)))
              lost(ws, "Link closed; retrying");
          } catch (Exception ignored) {
          }
        });
  }

  void optionsChanged() {
    try {
      boolean access = PhoneNotifications.access(this);
      emit(
          "deviceHello",
          new JSONObject()
              .put(
                  "capabilities",
                  new JSONObject()
                      .put("battery", true)
                      .put("charging", true)
                      .put("notifications", access)
                      .put("notificationBody", access && store.options().getBoolean("body", false))
                      .put("backgroundRelay", true)));
    } catch (Exception ignored) {
    }
  }

  private void sendBattery(Intent i) {
    if (i == null) return;
    try {
      int level = i.getIntExtra(BatteryManager.EXTRA_LEVEL, -1),
          scale = i.getIntExtra(BatteryManager.EXTRA_SCALE, -1),
          s = i.getIntExtra(BatteryManager.EXTRA_STATUS, -1);
      emit(
          "deviceState",
          new JSONObject()
              .put(
                  "batteryPct",
                  level < 0 || scale <= 0
                      ? JSONObject.NULL
                      : Math.max(0, Math.min(100, level * 100 / scale)))
              .put(
                  "charging",
                  s < 0
                      ? JSONObject.NULL
                      : s == BatteryManager.BATTERY_STATUS_CHARGING
                          || s == BatteryManager.BATTERY_STATUS_FULL));
    } catch (Exception ignored) {
    }
  }

  boolean connected() {
    return ready;
  }

  private void shutdown() {
    wakeLease.stop();
    if (ready && ws != null)
      try {
        ws.send(PhoneProtocol.event(++sequence, "sessionEnded", new JSONObject()));
      } catch (Exception ignored) {
      }
    running = false;
    ready = false;
    if (ws != null) {
      ws.close(1000, "User stopped");
      ws = null;
    }
    stopForeground(STOP_FOREGROUND_REMOVE);
    stopSelf();
  }

  public void onDestroy() {
    wakeLease.stop();
    if (ready && ws != null)
      try {
        ws.send(PhoneProtocol.event(++sequence, "sessionEnded", new JSONObject()));
        ws.close(1000, "User stopped");
      } catch (Exception ignored) {
      }
    running = false;
    ready = false;
    h.removeCallbacksAndMessages(null);
    discovery.stop();
    if (ws != null) ws.cancel();
    if (client != null) {
      client.connectionPool().evictAll();
      client.dispatcher().executorService().shutdown();
    }
    if (multicast != null && multicast.isHeld()) multicast.release();
    unregisterReceiver(battery);
    PhoneNotifications.clearTracked();
    current = null;
    if (status.startsWith("Connected")) status = "Stopped";
    super.onDestroy();
  }
}
