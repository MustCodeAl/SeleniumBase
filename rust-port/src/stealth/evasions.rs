//! JavaScript anti-detection / fingerprint spoofing payloads.
//!
//! The functions in this module build self-contained IIFE scripts from a
//! [`Fingerprint`](crate::stealth::fingerprint::Fingerprint). The returned
//! strings are suitable for:
//!
//! * `Page.addScriptToEvaluateOnNewDocument` via CDP (injected before any page
//!   JavaScript runs),
//! * `Page.evaluate` after navigation (rustwright / Playwright-compatible
//!   mode), or
//! * execution by a [`BrowserSession`](crate::browser::session::BrowserSession)
//!   helper.
//!
//! The evasions are inspired by `chromiumoxide_stealth`, `eoka`,
//! `chaser-oxide`, and `undetected-chromedriver`.

use std::collections::HashMap;

use crate::stealth::fingerprint::{
    CanvasNoiseMode, Fingerprint, MaskingMode, NoiseMode, OsType, PopupMode, ProxyMaskingMode,
};

/// Generates a single combined bootstrap script for the given fingerprint.
///
/// The script is wrapped in an IIFE and modifies prototype-level properties so
/// the spoofed values survive `getOwnPropertyNames` scans and re-define
/// attempts by page scripts.
pub fn bootstrap_script(fp: &Fingerprint) -> String {
    let mut parts = Vec::new();

    parts.push(framework_utils());
    parts.push(scrub_cdp_markers());

    if matches!(
        fp.flags.navigator_masking,
        MaskingMode::Mask | MaskingMode::Custom
    ) {
        parts.push(navigator_webdriver());
        parts.push(navigator_hardware(fp));
        parts.push(navigator_platform(fp));
        parts.push(navigator_plugins());
        parts.push(navigator_permissions());
        parts.push(navigator_languages(fp));
        parts.push(window_chrome());
    }

    if matches!(
        fp.flags.screen_masking,
        MaskingMode::Mask | MaskingMode::Custom
    ) {
        parts.push(screen_spoof(fp));
    }

    if matches!(
        fp.flags.graphics_masking,
        MaskingMode::Mask | MaskingMode::Custom
    ) {
        parts.push(webgl_vendor(fp));
    }

    if matches!(fp.flags.canvas_noise, CanvasNoiseMode::Mask) {
        parts.push(canvas_noise());
    }

    if matches!(
        fp.flags.audio_masking,
        MaskingMode::Mask | MaskingMode::Custom
    ) {
        parts.push(audio_noise());
    }

    if matches!(
        fp.flags.media_devices_masking,
        MaskingMode::Mask | MaskingMode::Custom
    ) {
        parts.push(media_devices(fp));
    }

    if matches!(
        fp.flags.timezone_masking,
        MaskingMode::Mask | MaskingMode::Custom
    ) {
        parts.push(timezone_spoof(fp));
    }

    if matches!(
        fp.flags.localization_masking,
        MaskingMode::Mask | MaskingMode::Custom
    ) {
        parts.push(localization_spoof(fp));
    }

    if matches!(
        fp.flags.graphics_noise,
        NoiseMode::Mask | NoiseMode::Natural
    ) {
        // Modernizr hairline fix helps headless detection tests.
        parts.push(hairline_fix());
    }

    parts.push(outer_dimensions(fp));
    parts.push(iframe_content_window());

    // NOTE: geolocation spoofing is best applied via CDP
    // `Emulation.setGeolocationOverride` or `--geolocation` flag.
    // We still patch `navigator.geolocation.getCurrentPosition` so that page JS
    // sees the requested coordinates when CDP overrides are unavailable.
    if matches!(
        fp.flags.geolocation_masking,
        MaskingMode::Mask | MaskingMode::Custom
    ) {
        parts.push(geolocation_spoof(fp));
    }

    if matches!(fp.flags.ports_masking, NoiseMode::Mask) {
        // Reserved for future port-list spoofing.
    }

    format!(
        "(() => {{
  'use strict';
  try {{
    {}
  }} catch (e) {{
    // Silently ignore failures so a broken evasion does not break the page.
  }}
}})();",
        parts.join("\n")
    )
}

/// Returns a script that scrubs `cdc_` / webdriver markers from `window` and
/// the prototype chain. Run inline on every navigation for WebDriver sessions.
pub fn cdc_scrub() -> String {
    "(() => {
  const re = /^[a-z]{3}_[a-zA-Z0-9]{22}_.*/;
  function scrub(obj) {
    if (!obj) return;
    Object.getOwnPropertyNames(obj).forEach(key => {
      if (re.test(key)) {
        try { delete obj[key]; } catch (e) {}
      }
    });
  }
  let o = window;
  while (o) {
    scrub(o);
    o = Object.getPrototypeOf(o);
  }
})();"
        .to_owned()
}

/// Utility helpers shared by several evasions.
fn framework_utils() -> String {
    "const _sbUtils = {
  cache: {
    Reflect: {
      get: Reflect.get.bind(Reflect),
      apply: Reflect.apply.bind(Reflect),
      defineProperty: Reflect.defineProperty.bind(Reflect),
    }
  },
  define(obj, prop, descriptor) {
    try {
      const result = this.cache.Reflect.defineProperty(obj, prop, descriptor);
      return result;
    } catch (e) {
      return false;
    }
  },
  addProxy(obj, propName, handler) {
    if (!obj || !obj.prototype) return;
    const prop = Object.getOwnPropertyDescriptor(obj.prototype, propName);
    if (!prop) return;
    try {
      Object.defineProperty(obj.prototype, propName, {
        ...prop,
        get: new Proxy(prop.get || function(){}, handler),
      });
    } catch (e) {}
  }
};"
    .to_owned()
}

/// Removes the primary `navigator.webdriver` flag.
fn navigator_webdriver() -> String {
    "delete Object.getPrototypeOf(navigator).webdriver;".to_owned()
}

/// Sets `navigator.hardwareConcurrency` and `navigator.deviceMemory`.
fn navigator_hardware(fp: &Fingerprint) -> String {
    let mut script = String::new();
    if let Some(cores) = fp.hardware_concurrency {
        script.push_str(&format!(
            "Object.defineProperty(Object.getPrototypeOf(navigator), 'hardwareConcurrency', {{ get() {{ return {cores}; }} }});"
        ));
    }
    if let Some(mem) = fp.device_memory {
        script.push_str(&format!(
            "Object.defineProperty(Object.getPrototypeOf(navigator), 'deviceMemory', {{ get() {{ return {mem}; }} }});"
        ));
    }
    if let Some(touch) = fp.max_touch_points {
        script.push_str(&format!(
            "Object.defineProperty(Object.getPrototypeOf(navigator), 'maxTouchPoints', {{ get() {{ return {touch}; }} }});"
        ));
    }
    script
}

/// Sets `navigator.platform` and, if provided, `navigator.userAgent`.
fn navigator_platform(fp: &Fingerprint) -> String {
    let platform = fp
        .platform
        .as_deref()
        .unwrap_or_else(|| fp.os_type.platform());
    let mut script = format!(
        "Object.defineProperty(Object.getPrototypeOf(navigator), 'platform', {{ get() {{ return '{platform}'; }} }});"
    );
    if let Some(ua) = fp.user_agent.as_deref() {
        let escaped = ua.replace('\\', "\\\\").replace('\'', "\\'");
        script.push_str(&format!(
            "Object.defineProperty(Object.getPrototypeOf(navigator), 'userAgent', {{ get() {{ return '{escaped}'; }} }});"
        ));
    }
    script
}

/// Injects a fake plugin list.
fn navigator_plugins() -> String {
    "(function() {
  function makePlugins() {
    function FakePlugins(length) {
      Object.setPrototypeOf(this, PluginArray.prototype);
      for (let i = 0; i < length; i++) {
        this[i] = {
          name: 'Plugin ' + i,
          filename: 'plugin' + i + '.so',
          description: 'plugin ' + i,
          version: undefined,
          length: 1,
          item(idx) { return this[idx]; },
          namedItem(name) { return null; },
        };
      }
      this.length = length;
      this.item = function(idx) { return this[idx]; };
      this.namedItem = function(name) { return null; };
      this.refresh = function() {};
    }
    FakePlugins.prototype = PluginArray.prototype;
    return new FakePlugins(3);
  }
  Object.defineProperty(Object.getPrototypeOf(navigator), 'plugins', { get: makePlugins });

  function makeMimeTypes() {
    function FakeMimeTypes(length) {
      Object.setPrototypeOf(this, MimeTypeArray.prototype);
      for (let i = 0; i < length; i++) {
        this[i] = {
          type: 'application/x-' + i,
          suffixes: 'x' + i,
          description: 'mime ' + i,
          enabledPlugin: null,
        };
      }
      this.length = length;
      this.item = function(idx) { return this[idx]; };
      this.namedItem = function(name) { return null; };
    }
    FakeMimeTypes.prototype = MimeTypeArray.prototype;
    return new FakeMimeTypes(2);
  }
  Object.defineProperty(Object.getPrototypeOf(navigator), 'mimeTypes', { get: makeMimeTypes });
})();"
        .to_owned()
}

/// Fixes `navigator.permissions.query('notifications')` so it does not return
/// the headless default of `prompt`/`denied`.
fn navigator_permissions() -> String {
    "(function() {
  const orig = navigator.permissions.query;
  navigator.permissions.query = function(parameters) {
    if (parameters && parameters.name === 'notifications') {
      return Promise.resolve({
        state: Notification.permission,
        onchange: null,
        addEventListener: function() {},
        removeEventListener: function() {},
        dispatchEvent: function() { return true; },
      });
    }
    return orig.call(this, parameters);
  };
})();"
        .to_owned()
}

/// Sets `navigator.languages`.
fn navigator_languages(fp: &Fingerprint) -> String {
    let langs = fp
        .languages
        .as_deref()
        .map(|s| s.to_owned())
        .unwrap_or_else(|| "en-US,en".to_owned());
    let list: Vec<&str> = langs.split(',').map(str::trim).collect();
    let primary = list.first().copied().unwrap_or("en-US");
    let array = list
        .iter()
        .map(|s| format!("'{s}'"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "Object.defineProperty(Object.getPrototypeOf(navigator), 'language', {{ get() {{ return '{primary}'; }} }});
Object.defineProperty(Object.getPrototypeOf(navigator), 'languages', {{ get() {{ return [{array}]; }} }});"
    )
}

/// Mocks `window.chrome.runtime` and `window.chrome.app`.
fn window_chrome() -> String {
    "(function() {
  window.chrome = window.chrome || {};
  window.chrome.app = {
    isInstalled: false,
    InstallState: { DISABLED: 'disabled', INSTALLED: 'installed', NOT_INSTALLED: 'not_installed' },
    RunningState: { CANNOT_RUN: 'cannot_run', READY_TO_RUN: 'ready_to_run', RUNNING: 'running' },
    getDetails: function() {},
    getIsInstalled: function() { return false; },
    installState: function() { return Promise.resolve('not_installed'); },
    runningState: function() { return 'cannot_run'; },
  };
  window.chrome.runtime = {
    id: undefined,
    OnInstalledReason: { CHROME_UPDATE: 'chrome_update', INSTALL: 'install', SHARED_MODULE_UPDATE: 'shared_module_update', UPDATE: 'update' },
    OnRestartRequiredReason: { APP_UPDATE: 'app_update', OS_UPDATE: 'os_update', PERIODIC: 'periodic' },
    PlatformArch: { ARM: 'arm', ARM64: 'arm64', MIPS: 'mips', MIPS64: 'mips64', X86_32: 'x86-32', X86_64: 'x86-64' },
    PlatformNaclArch: { ARM: 'arm', MIPS: 'mips', MIPS64: 'mips64', X86_32: 'x86-32', X86_64: 'x86-64' },
    PlatformOs: { ANDROID: 'android', CROS: 'cros', LINUX: 'linux', MAC: 'mac', OPENBSD: 'openbsd', WIN: 'win' },
    RequestUpdateCheckStatus: { NO_UPDATE: 'no_update', THROTTLED: 'throttled', UPDATE_AVAILABLE: 'update_available' },
    connect: function() {
      return { postMessage: function(){}, disconnect: function(){}, onDisconnect: { addListener: function(){} }, onMessage: { addListener: function(){} } };
    },
    sendMessage: function() {
      if (arguments.length > 0 && typeof arguments[arguments.length - 1] === 'function') {
        arguments[arguments.length - 1]({});
      }
      return Promise.resolve({});
    },
    getURL: function(path) { return 'chrome-extension://' + (this.id || '') + (path || ''); },
    getManifest: function() { return {}; },
    getPlatformInfo: function() { return Promise.resolve({ os: 'win', arch: 'x86-64', nacl_arch: 'x86-64' }); },
    onConnect: { addListener: function(){}, removeListener: function(){}, hasListener: function(){ return false; } },
    onMessage: { addListener: function(){}, removeListener: function(){}, hasListener: function(){ return false; } },
  };
})();"
    .to_owned()
}

/// Spoofs `screen.*` dimensions.
fn screen_spoof(fp: &Fingerprint) -> String {
    let (w, h) = match (fp.screen_width, fp.screen_height) {
        (Some(w), Some(h)) => (w, h),
        _ => fp.os_type.default_screen(),
    };
    let pr = fp.pixel_ratio.unwrap_or(1.0);
    let depth = fp.color_depth.unwrap_or(24);
    let avail_w = w;
    let avail_h = h - 40; // rough taskbar allowance

    format!(
        "Object.defineProperty(window.Screen.prototype, 'width', {{ get() {{ return {w}; }} }});
Object.defineProperty(window.Screen.prototype, 'height', {{ get() {{ return {h}; }} }});
Object.defineProperty(window.Screen.prototype, 'availWidth', {{ get() {{ return {avail_w}; }} }});
Object.defineProperty(window.Screen.prototype, 'availHeight', {{ get() {{ return {avail_h}; }} }});
Object.defineProperty(window.Screen.prototype, 'colorDepth', {{ get() {{ return {depth}; }} }});
Object.defineProperty(window.Screen.prototype, 'pixelDepth', {{ get() {{ return {depth}; }} }});
Object.defineProperty(window.visualViewport || window, 'width', {{ get() {{ return {w}; }} }}, {{ configurable: true }});
Object.defineProperty(window.visualViewport || window, 'height', {{ get() {{ return {h}; }} }}, {{ configurable: true }});
window.devicePixelRatio = {pr};"
    )
}

/// Overrides WebGL `UNMASKED_VENDOR_WEBGL` and `UNMASKED_RENDERER_WEBGL`.
fn webgl_vendor(fp: &Fingerprint) -> String {
    let vendor = fp.webgl_vendor.as_deref().unwrap_or("Intel Inc.");
    let renderer = fp
        .webgl_renderer
        .as_deref()
        .unwrap_or("Intel Iris OpenGL Engine");
    let vendor_escaped = vendor.replace('\\', "\\\\").replace('\'', "\\'");
    let renderer_escaped = renderer.replace('\\', "\\\\").replace('\'', "\\'");

    format!(
        "(function() {{
  const vendor = '{vendor_escaped}';
  const renderer = '{renderer_escaped}';
  function patch(ctx) {{
    if (!ctx || !ctx.prototype) return;
    const orig = ctx.prototype.getParameter;
    ctx.prototype.getParameter = function(parameter) {{
      if (parameter === 37445) return vendor;
      if (parameter === 37446) return renderer;
      return orig.call(this, parameter);
    }};
  }}
  patch(window.WebGLRenderingContext);
  patch(window.WebGL2RenderingContext);
}})();"
    )
}

/// Adds subtle deterministic noise to `HTMLCanvasElement.toDataURL` /
/// `getImageData` so repeated identical draws do not produce identical pixels.
fn canvas_noise() -> String {
    "(function() {
  function noise() { return Math.floor(Math.random() * 3) - 1; }
  const orig = HTMLCanvasElement.prototype.toDataURL;
  HTMLCanvasElement.prototype.toDataURL = function(...args) {
    try {
      const ctx = this.getContext('2d');
      if (ctx && this.width > 0 && this.height > 0) {
        const img = ctx.getImageData(0, 0, this.width, this.height);
        for (let i = 0; i < img.data.length; i += 4) {
          img.data[i] = Math.max(0, Math.min(255, img.data[i] + noise()));
        }
        ctx.putImageData(img, 0, 0);
      }
    } catch (e) {}
    return orig.apply(this, args);
  };
})();"
        .to_owned()
}

/// Adds tiny noise to `AnalyserNode.getFloatFrequencyData`.
fn audio_noise() -> String {
    "(function() {
  const orig = window.AnalyserNode && window.AnalyserNode.prototype.getFloatFrequencyData;
  if (!orig) return;
  window.AnalyserNode.prototype.getFloatFrequencyData = function(array) {
    orig.call(this, array);
    for (let i = 0; i < array.length; i++) {
      array[i] += (Math.random() - 0.5) * 0.0002;
    }
  };
})();"
        .to_owned()
}

/// Spoofs `navigator.mediaDevices.enumerateDevices` counts.
fn media_devices(fp: &Fingerprint) -> String {
    let audio_in = fp.audio_inputs.unwrap_or(1);
    let audio_out = fp.audio_outputs.unwrap_or(1);
    let video_in = fp.video_inputs.unwrap_or(1);

    format!(
        "(function() {{
  if (!navigator.mediaDevices) return;
  const labels = ['Default', 'Communications', 'Microphone', 'Camera', 'Speaker'];
  const orig = navigator.mediaDevices.enumerateDevices.bind(navigator.mediaDevices);
  navigator.mediaDevices.enumerateDevices = function() {{
    return orig().then(devices => {{
      const out = [];
      for (let i = 0; i < {audio_in}; i++) out.push({{ deviceId: 'audioinput-' + i, groupId: 'grp-' + i, kind: 'audioinput', label: labels[i % labels.length] }});
      for (let i = 0; i < {audio_out}; i++) out.push({{ deviceId: 'audiooutput-' + i, groupId: 'grp-' + i, kind: 'audiooutput', label: labels[i % labels.length] }});
      for (let i = 0; i < {video_in}; i++) out.push({{ deviceId: 'videoinput-' + i, groupId: 'grp-' + i, kind: 'videoinput', label: 'Camera ' + i }});
      return out;
    }});
  }};
}})();"
    )
}

/// Spoofs the time zone used by `Intl.DateTimeFormat` and `Date` methods.
fn timezone_spoof(fp: &Fingerprint) -> String {
    let zone = fp.timezone.as_deref().unwrap_or("America/New_York");
    let zone_escaped = zone.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        "(function() {{
  const zone = '{zone_escaped}';
  const orig = Intl.DateTimeFormat;
  Intl.DateTimeFormat = function(...args) {{
    if (!args[1]) args[1] = {{}};
    args[1].timeZone = zone;
    return new orig(...args);
  }};
  Intl.DateTimeFormat.supportedLocalesOf = orig.supportedLocalesOf;
}})();"
    )
}

/// Spoofs localization-related values.
fn localization_spoof(fp: &Fingerprint) -> String {
    let locale = fp.locale.as_deref().unwrap_or("en-US");
    let accept = fp.accept_languages.as_deref().unwrap_or("en-US,en;q=0.9");
    let locale_escaped = locale.replace('\\', "\\\\").replace('\'', "\\'");
    let accept_escaped = accept.replace('\\', "\\\\").replace('\'', "\\'");

    format!(
        "Object.defineProperty(Object.getPrototypeOf(navigator), 'locale', {{ get() {{ return '{locale_escaped}'; }} }});
// Some detectors read the Accept-Language hint from a custom property.
window.navigator.__acceptLanguages = '{accept_escaped}';"
    )
}

/// Fixes `window.outerWidth` / `outerHeight` leak in headless Chrome.
fn outer_dimensions(fp: &Fingerprint) -> String {
    let (w, h) = match (fp.screen_width, fp.screen_height) {
        (Some(w), Some(h)) => (w, h),
        _ => fp.os_type.default_screen(),
    };
    format!(
        "Object.defineProperty(window, 'outerWidth', {{ get() {{ return {w}; }} }});
Object.defineProperty(window, 'outerHeight', {{ get() {{ return {h}; }} }});"
    )
}

/// Fixes iframe `contentWindow` consistency checks.
fn iframe_content_window() -> String {
    "(function() {
  const orig = document.createElement;
  document.createElement = function(tagName, options) {
    const el = orig.call(document, tagName, options);
    if (tagName && tagName.toLowerCase() === 'iframe') {
      try {
        const contentWindow = el.contentWindow;
        if (contentWindow) {
          Object.defineProperty(contentWindow, 'self', { get() { return contentWindow; } });
          Object.defineProperty(contentWindow, 'window', { get() { return contentWindow; } });
        }
      } catch (e) {}
    }
    return el;
  };
})();"
        .to_owned()
}

/// Fixes Modernizr hairline headless detection.
fn hairline_fix() -> String {
    "(function() {
  const desc = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'offsetHeight');
  if (!desc) return;
  Object.defineProperty(HTMLDivElement.prototype, 'offsetHeight', {
    ...desc,
    get: function() {
      if (this.id === 'modernizr') return 1;
      return desc.get.apply(this);
    }
  });
})();"
        .to_owned()
}

/// Scrubs CDP markers such as `cdc_` and `__webdriver` from `window`.
fn scrub_cdp_markers() -> String {
    "(function() {
  const patterns = [/^cdc_[a-zA-Z0-9]{22}_/, /^\\$cdc_[a-zA-Z0-9]{22}_/, /__webdriver/, /__selenium/, /__driver/, /\\$chrome_/];
  function scrub(obj) {
    if (!obj) return;
    Object.getOwnPropertyNames(obj).forEach(key => {
      if (patterns.some(p => p.test(key))) {
        try { delete obj[key]; } catch (e) {}
      }
    });
  }
  let o = window;
  while (o) { scrub(o); o = Object.getPrototypeOf(o); }
})();"
    .to_owned()
}

/// Spoofs `navigator.geolocation.getCurrentPosition` when CDP overrides are
/// unavailable.
fn geolocation_spoof(fp: &Fingerprint) -> String {
    let lat = fp.latitude.unwrap_or(0.0);
    let lon = fp.longitude.unwrap_or(0.0);
    let alt = fp.altitude.unwrap_or(0.0);
    let acc = fp.accuracy.unwrap_or(100.0);

    let popup = match fp.flags.geolocation_popup {
        PopupMode::Allow => "true",
        _ => "false",
    };

    format!(
        "(function() {{
  if (!navigator.geolocation) return;
  const coords = {{
    latitude: {lat},
    longitude: {lon},
    altitude: {alt},
    accuracy: {acc},
    altitudeAccuracy: {acc},
    heading: null,
    speed: null,
  }};
  const pos = {{ coords, timestamp: Date.now() }};
  const orig = navigator.geolocation.getCurrentPosition.bind(navigator.geolocation);
  navigator.geolocation.getCurrentPosition = function(success, error, options) {{
    if ({popup}) {{
      setTimeout(() => success && success(pos), 0);
      return;
    }}
    return orig(success, error, options);
  }};
}})();"
    )
}

/// Returns Chromium command-line arguments derived from a fingerprint.
pub fn launch_args(fp: &Fingerprint) -> Vec<String> {
    let mut args = vec![
        "--disable-blink-features=AutomationControlled".to_owned(),
        "--disable-infobars".to_owned(),
        "--no-default-browser-check".to_owned(),
        "--no-first-run".to_owned(),
        "--no-service-autorun".to_owned(),
        "--no-pings".to_owned(),
        "--disable-background-timer-throttling".to_owned(),
        "--disable-backgrounding-occluded-windows".to_owned(),
        "--disable-renderer-backgrounding".to_owned(),
        "--disable-features=IsolateOrigins,site-per-process,PrivacySandboxSettings4".to_owned(),
    ];

    if let Some(ua) = fp.user_agent.as_deref() {
        args.push(format!("--user-agent={ua}"));
    }

    if let Some(locale) = fp.locale.as_deref() {
        args.push(format!("--lang={locale}"));
    }

    if let (Some(w), Some(h)) = (fp.screen_width, fp.screen_height) {
        args.push(format!("--window-size={w},{h}"));
    }

    match fp.webrtc_policy {
        crate::stealth::fingerprint::WebRtcPolicy::DisableNonProxiedUdp => {
            args.push("--force-webrtc-ip-handling-policy=disable_non_proxied_udp".to_owned());
        }
        crate::stealth::fingerprint::WebRtcPolicy::PublicInterfaceOnly => {
            args.push("--force-webrtc-ip-handling-policy=default_public_interface_only".to_owned());
        }
        crate::stealth::fingerprint::WebRtcPolicy::PublicAndPrivateInterfaces => {
            args.push(
                "--force-webrtc-ip-handling-policy=default_public_and_private_interfaces"
                    .to_owned(),
            );
        }
    }

    if matches!(fp.flags.proxy_masking, ProxyMaskingMode::Custom) {
        if let Some(proxy) = fp.proxy.as_ref() {
            args.push(format!("--proxy-server={}", proxy.to_url()));
        }
    }

    if fp.flags.quic_mode == crate::stealth::fingerprint::QuicMode::Disabled {
        args.push("--disable-quic".to_owned());
    }

    for (flag, value) in &fp.cmd_params {
        if value.is_empty() {
            args.push(format!("--{flag}"));
        } else {
            args.push(format!("--{flag}={value}"));
        }
    }

    args
}

/// Returns CDP parameter overrides recommended for a fingerprint.
pub fn cdp_overrides(fp: &Fingerprint) -> HashMap<String, serde_json::Value> {
    let mut map = HashMap::new();

    if let (Some(lat), Some(lon)) = (fp.latitude, fp.longitude) {
        map.insert(
            "Emulation.setGeolocationOverride".to_owned(),
            serde_json::json!({
                "latitude": lat,
                "longitude": lon,
                "accuracy": fp.accuracy.unwrap_or(100.0),
                "altitude": fp.altitude.unwrap_or(0.0),
            }),
        );
    }

    if let (Some(w), Some(h)) = (fp.screen_width, fp.screen_height) {
        map.insert(
            "Emulation.setDeviceMetricsOverride".to_owned(),
            serde_json::json!({
                "width": w,
                "height": h,
                "deviceScaleFactor": fp.pixel_ratio.unwrap_or(1.0),
                "mobile": fp.os_type == OsType::Android,
            }),
        );
    }

    if let Some(ua) = fp.user_agent.as_deref() {
        map.insert(
            "Network.setUserAgentOverride".to_owned(),
            serde_json::json!({ "userAgent": ua }),
        );
    }

    if let Some(tz) = fp.timezone.as_deref() {
        map.insert(
            "Emulation.setTimezoneOverride".to_owned(),
            serde_json::json!({ "timezoneId": tz }),
        );
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stealth::fingerprint::Fingerprint;

    #[test]
    fn bootstrap_includes_webdriver_evasion() {
        let fp = Fingerprint::windows_desktop();
        let script = bootstrap_script(&fp);
        assert!(script.contains("webdriver"));
        assert!(script.contains("Google Inc. (NVIDIA)"));
        assert!(script.contains("1920"));
    }

    #[test]
    fn launch_args_include_stealth_flags() {
        let fp = Fingerprint::windows_desktop();
        let args = launch_args(&fp);
        assert!(args.iter().any(|a| a.contains("AutomationControlled")));
        assert!(args.iter().any(|a| a.starts_with("--user-agent=")));
        assert!(args.iter().any(|a| a.starts_with("--window-size=")));
    }

    #[test]
    fn cdp_overrides_contain_screen_and_geo() {
        let fp = Fingerprint::builder()
            .screen(1920, 1080)
            .geolocation(40.0, -74.0)
            .build();
        let map = cdp_overrides(&fp);
        assert!(map.contains_key("Emulation.setDeviceMetricsOverride"));
        assert!(map.contains_key("Emulation.setGeolocationOverride"));
    }
}
