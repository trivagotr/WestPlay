package app.westplay.phone;

import android.app.*;
import android.content.*;
import android.graphics.Color;
import android.os.*;
import android.provider.Settings;
import android.text.InputType;
import android.widget.*;
import com.google.zxing.integration.android.*;
import org.json.*;

public final class MainActivity extends Activity {
  private EditText link, filters;
  private TextView status;
  private CredentialStore store;
  private final Handler h = new Handler(Looper.getMainLooper());
  private final Runnable refresh =
      new Runnable() {
        public void run() {
          if (status != null) {
            boolean access = PhoneNotifications.access(MainActivity.this);
            status.setText(
                RelayService.status
                    + "\nNotification access: "
                    + (access ? "enabled" : "off")
                    + " • body: "
                    + (store.options().getBoolean("body", false) ? "shared" : "private"));
          }
          h.postDelayed(this, 1000);
        }
      };

  public void onCreate(Bundle b) {
    super.onCreate(b);
    store = new CredentialStore(this);
    ScrollView sc = new ScrollView(this);
    LinearLayout page = new LinearLayout(this);
    page.setOrientation(LinearLayout.VERTICAL);
    page.setPadding(24, 36, 24, 28);
    sc.addView(page);
    setContentView(sc);
    TextView title = new TextView(this);
    title.setText("WESTPLAY / PHONE");
    title.setTextSize(26);
    title.setTextColor(Color.rgb(104, 230, 148));
    page.addView(title);
    text(
        page,
        "Keep your phone battery and chosen notifications in your PC game. Connect on the same"
            + " local network.");
    status = new TextView(this);
    status.setTextSize(16);
    page.addView(status);
    link = new EditText(this);
    link.setHint("Paste westplay://phone pairing link");
    link.setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_VARIATION_URI);
    link.setSingleLine(false);
    page.addView(link);
    if (getIntent().getData() != null) link.setText(getIntent().getDataString());
    button(
        page,
        "Scan PC QR",
        () ->
            new IntentIntegrator(this)
                .setDesiredBarcodeFormats(IntentIntegrator.QR_CODE)
                .setPrompt("Scan WestPlay phone data QR")
                .setBeepEnabled(false)
                .setOrientationLocked(false)
                .initiateScan());
    button(page, "Connect / reconnect known PC", () -> connect());
    button(
        page,
        "Stop link",
        () -> {
          stopService(new Intent(this, RelayService.class));
          RelayService.status = "Stopped";
        });
    button(
        page,
        "Notification access settings",
        () -> startActivity(new Intent(Settings.ACTION_NOTIFICATION_LISTENER_SETTINGS)));
    Switch body = new Switch(this);
    body.setText("Share notification body (default off)");
    body.setChecked(store.options().getBoolean("body", false));
    body.setOnCheckedChangeListener(
        (v, on) -> {
          store.options().edit().putBoolean("body", on).apply();
          PhoneNotifications.clearTracked();
          if (RelayService.current != null) RelayService.current.optionsChanged();
        });
    page.addView(body);
    text(
        page,
        "Blocked apps — package names separated by commas. Notifications from WestPlay,"
            + " ongoing/media/progress and secret content are excluded. Private notifications"
            + " received while locked are also excluded.");
    filters = new EditText(this);
    filters.setHint("com.example.bank, com.example.private");
    filters.setText(store.options().getString("blocked", ""));
    page.addView(filters);
    button(
        page,
        "Save app filters",
        () -> {
          store.options().edit().putString("blocked", filters.getText().toString().trim()).apply();
          PhoneNotifications.clearTracked();
          toast("Filters saved");
        });
    button(
        page,
        "Send test event",
        () -> {
          RelayService s = RelayService.current;
          if (s == null || !s.connected()) {
            toast("Connect first");
            return;
          }
          try {
            s.emit(
                "notificationPosted",
                new JSONObject()
                    .put("id", "westplay-test")
                    .put("appId", getPackageName())
                    .put("appName", "WestPlay Phone")
                    .put("title", "Phone link test")
                    .put("body", JSONObject.NULL));
            toast("Test event sent");
          } catch (Exception ignored) {
          }
        });
    button(
        page,
        "App background / battery settings",
        () ->
            startActivity(
                new Intent(
                    Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
                    android.net.Uri.parse("package:" + getPackageName()))));
    text(
        page,
        "Start the link here before turning the screen off. The ongoing foreground notification"
            + " keeps the connected-device service visible. Android or your phone vendor may still"
            + " suspend network access in Doze or restricted battery mode. If needed, review this"
            + " app's battery setting yourself. No permissions or battery restrictions are changed"
            + " automatically. Stop ends the background link. Reopen and reconnect after a reboot"
            + " or force-stop.");
    button(
        page,
        "Forget paired PC",
        () ->
            new AlertDialog.Builder(this)
                .setTitle("Forget this PC?")
                .setMessage(
                    "The local credential and saved PC key will be removed. Pair again with a fresh"
                        + " PC QR.")
                .setNegativeButton("Cancel", null)
                .setPositiveButton(
                    "Forget",
                    (d, w) -> {
                      stopService(new Intent(this, RelayService.class));
                      store.forget();
                      link.setText("");
                      RelayService.status = "PC forgotten";
                    })
                .show());
  }

  private void connect() {
    try {
      String value = link.getText().toString().trim();
      if (!value.isEmpty()) PhoneProtocol.parse(value);
      if (Build.VERSION.SDK_INT >= 33
          && checkSelfPermission(android.Manifest.permission.POST_NOTIFICATIONS)
              != android.content.pm.PackageManager.PERMISSION_GRANTED)
        requestPermissions(new String[] {android.Manifest.permission.POST_NOTIFICATIONS}, 1);
      Intent i = new Intent(this, RelayService.class);
      if (!value.isEmpty()) i.putExtra("uri", value);
      startForegroundService(i);
      link.setText("");
    } catch (Exception e) {
      toast(e.getMessage() == null ? "Unable to connect" : e.getMessage());
    }
  }

  private void button(LinearLayout l, String title, Runnable action) {
    Button b = new Button(this);
    b.setText(title);
    b.setAllCaps(false);
    b.setOnClickListener(v -> action.run());
    l.addView(b);
  }

  private void text(LinearLayout l, String value) {
    TextView t = new TextView(this);
    t.setText(value);
    t.setPadding(0, 16, 0, 16);
    l.addView(t);
  }

  private void toast(String s) {
    Toast.makeText(this, s, Toast.LENGTH_LONG).show();
  }

  protected void onResume() {
    super.onResume();
    h.post(refresh);
    if (RelayService.current != null) RelayService.current.optionsChanged();
  }

  protected void onPause() {
    h.removeCallbacks(refresh);
    super.onPause();
  }

  protected void onNewIntent(Intent i) {
    super.onNewIntent(i);
    setIntent(i);
    if (i.getData() != null) link.setText(i.getDataString());
  }

  protected void onActivityResult(int request, int result, Intent data) {
    IntentResult r = IntentIntegrator.parseActivityResult(request, result, data);
    if (r != null) {
      if (r.getContents() != null) link.setText(r.getContents());
      return;
    }
    super.onActivityResult(request, result, data);
  }
}
