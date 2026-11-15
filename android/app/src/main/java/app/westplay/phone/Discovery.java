package app.westplay.phone;

import android.content.Context;
import android.net.nsd.*;
import android.os.*;
import java.nio.charset.StandardCharsets;
import java.util.function.Consumer;

final class Discovery {
  private final NsdManager nsd;
  private final Handler handler = new Handler(Looper.getMainLooper());
  private NsdManager.DiscoveryListener listener;
  private final ResolutionQueue<NsdServiceInfo> resolutions;

  Discovery(Context c) {
    nsd = (NsdManager) c.getSystemService(Context.NSD_SERVICE);
    resolutions =
        new ResolutionQueue<>(
            64,
            (service, completed) ->
                nsd.resolveService(
                    service,
                    new NsdManager.ResolveListener() {
                      public void onResolveFailed(NsdServiceInfo s, int e) {
                        handler.post(() -> completed.accept(null));
                      }

                      public void onServiceResolved(NsdServiceInfo s) {
                        handler.post(() -> completed.accept(s));
                      }
                    }));
  }

  void start(String pc, Consumer<NsdServiceInfo> found) {
    stop();
    long generation =
        resolutions.start(
            s -> {
              var a = s.getAttributes();
              if (pc.equals(txt(a.get("pcId")))
                  && "1".equals(txt(a.get("v")))
                  && "phone-data".equals(txt(a.get("role")))
                  && s.getHost() != null
                  && PhoneProtocol.privateIp(s.getHost().getHostAddress())
                  && s.getPort() > 0
                  && s.getPort() <= 65535) found.accept(s);
            });
    listener =
        new NsdManager.DiscoveryListener() {
          public void onDiscoveryStarted(String t) {}

          public void onDiscoveryStopped(String t) {}

          public void onServiceLost(NsdServiceInfo s) {
            handler.post(() -> resolutions.remove(generation, key(s)));
          }

          public void onStartDiscoveryFailed(String t, int e) {
            var source = this;
            handler.post(
                () -> {
                  if (listener == source) stop();
                });
          }

          public void onStopDiscoveryFailed(String t, int e) {}

          public void onServiceFound(NsdServiceInfo s) {
            if (s.getServiceType() != null && s.getServiceType().contains("_westplay._tcp"))
              handler.post(() -> resolutions.offer(generation, key(s), s));
          }
        };
    try {
      nsd.discoverServices("_westplay._tcp.", NsdManager.PROTOCOL_DNS_SD, listener);
    } catch (Exception e) {
      stop();
    }
  }

  private static String key(NsdServiceInfo s) {
    return s.getServiceName() + "|" + s.getServiceType();
  }

  private static String txt(byte[] b) {
    return b == null ? "" : new String(b, StandardCharsets.UTF_8);
  }

  void stop() {
    resolutions.stop();
    if (listener != null) {
      var previous = listener;
      listener = null;
      try {
        nsd.stopServiceDiscovery(previous);
      } catch (Exception ignored) {
      }
    }
  }
}
