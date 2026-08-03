const { invoke } = window.__TAURI__.core;

const statusEl = document.querySelector("#status");

function setStatus(msg) {
  statusEl.textContent = msg;
}

async function refreshProfiles() {
  const profiles = await invoke("list_profiles");
  const list = document.querySelector("#profile-list");
  list.innerHTML = "";
  for (const p of profiles) {
    const li = document.createElement("li");
    const geo = p.latitude != null && p.longitude != null
      ? `📍 ${p.latitude.toFixed(4)}, ${p.longitude.toFixed(4)}`
      : "";
    li.innerHTML = `
      <div class="profile-row">
        <strong>${p.name}</strong>
        <span class="muted">${p.container_url}</span>
        <span class="tag">${p.mode || "webdriver"}</span>
        <span class="muted">${geo}</span>
      </div>
      <div class="actions">
        <input type="text" class="start-url" placeholder="Start URL" />
        <button class="launch">Launch</button>
        <button class="delete">Delete</button>
      </div>
    `;
    li.querySelector(".launch").addEventListener("click", async () => {
      const url = li.querySelector(".start-url").value || undefined;
      setStatus(`Launching ${p.name}...`);
      try {
        const info = await invoke("launch_profile", { id: p.id, startUrl: url });
        setStatus(`Launched ${info.profile_name} → ${info.session_id}`);
        refreshSessions();
      } catch (e) {
        setStatus(`Launch failed: ${e}`);
      }
    });
    li.querySelector(".delete").addEventListener("click", async () => {
      await invoke("delete_profile", { id: p.id });
      refreshProfiles();
    });
    list.appendChild(li);
  }
}

async function apiBase() {
  return invoke("get_api_base");
}

async function refreshTags() {
  const base = await apiBase();
  try {
    const [tagsRes, foldersRes] = await Promise.all([
      fetch(`${base}/api/v1/tags`),
      fetch(`${base}/api/v1/folders`),
    ]);
    const tagsBody = await tagsRes.json();
    const foldersBody = await foldersRes.json();

    const tagList = document.querySelector("#tag-list");
    tagList.innerHTML = "";
    for (const t of tagsBody.data || []) {
      const li = document.createElement("li");
      li.textContent = `${t.name} (${t.color})`;
      tagList.appendChild(li);
    }

    const folderList = document.querySelector("#folder-list");
    folderList.innerHTML = "";
    for (const f of foldersBody.data || []) {
      const li = document.createElement("li");
      li.textContent = f.name;
      folderList.appendChild(li);
    }
  } catch (e) {
    setStatus(`Tags refresh failed: ${e}`);
  }
}

async function refreshSessions() {
  const sessions = await invoke("list_sessions");
  const list = document.querySelector("#session-list");
  list.innerHTML = "";
  for (const s of sessions) {
    const li = document.createElement("li");
    li.innerHTML = `
      <div class="session-row">
        <strong>${s.profile_name}</strong>
        <span class="muted">${s.container_url}</span>
      </div>
      <div class="actions">
        <input type="text" class="nav-url" placeholder="Navigate to URL" />
        <button class="navigate">Go</button>
        <button class="screenshot">Screenshot</button>
      </div>
      <div class="actions">
        <input type="number" step="any" class="lat" placeholder="Lat" />
        <input type="number" step="any" class="lon" placeholder="Lon" />
        <input type="number" step="any" class="acc" placeholder="Accuracy m" />
        <button class="set-geo">Set Geo</button>
        <button class="close">Close</button>
      </div>
    `;
    li.querySelector(".navigate").addEventListener("click", async () => {
      const url = li.querySelector(".nav-url").value;
      if (!url) return;
      setStatus(`Navigating ${s.session_id}...`);
      try {
        await invoke("navigate_session", { sessionId: s.session_id, url });
        setStatus("Navigation complete");
      } catch (e) {
        setStatus(`Navigation failed: ${e}`);
      }
    });
    li.querySelector(".screenshot").addEventListener("click", async () => {
      setStatus("Taking screenshot...");
      try {
        const path = await invoke("take_screenshot", { sessionId: s.session_id });
        setStatus(`Screenshot saved: ${path}`);
      } catch (e) {
        setStatus(`Screenshot failed: ${e}`);
      }
    });
    li.querySelector(".set-geo").addEventListener("click", async () => {
      const lat = Number(li.querySelector(".lat").value);
      const lon = Number(li.querySelector(".lon").value);
      const acc = li.querySelector(".acc").value;
      if (Number.isNaN(lat) || Number.isNaN(lon)) {
        setStatus("Enter valid latitude and longitude");
        return;
      }
      setStatus(`Setting geolocation ${lat}, ${lon}...`);
      try {
        await invoke("set_session_geolocation", {
          sessionId: s.session_id,
          latitude: lat,
          longitude: lon,
          accuracy: acc ? Number(acc) : undefined,
        });
        setStatus("Geolocation updated");
      } catch (e) {
        setStatus(`Geolocation failed: ${e}`);
      }
    });
    li.querySelector(".close").addEventListener("click", async () => {
      await invoke("close_session", { sessionId: s.session_id });
      refreshSessions();
    });
    list.appendChild(li);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#profile-form").addEventListener("submit", async (e) => {
    e.preventDefault();
    const parseNum = (id) => {
      const el = document.querySelector(id);
      const v = el.value.trim();
      return v ? Number(v) : undefined;
    };
    const newProfile = {
      name: document.querySelector("#profile-name").value,
      containerUrl: document.querySelector("#container-url").value,
      userAgent: document.querySelector("#user-agent").value || undefined,
      proxy: document.querySelector("#proxy").value || undefined,
      locale: document.querySelector("#locale").value || undefined,
      latitude: parseNum("#latitude"),
      longitude: parseNum("#longitude"),
      accuracy: parseNum("#accuracy"),
      headless: document.querySelector("#headless").checked,
    };
    await invoke("create_profile", { new: newProfile });
    e.target.reset();
    refreshProfiles();
  });

  document.querySelector("#refresh-profiles").addEventListener("click", refreshProfiles);
  document.querySelector("#refresh-sessions").addEventListener("click", refreshSessions);
  document.querySelector("#refresh-tags").addEventListener("click", refreshTags);

  document.querySelector("#add-tag").addEventListener("click", async () => {
    const name = document.querySelector("#new-tag").value;
    if (!name) return;
    const base = await apiBase();
    try {
      await fetch(`${base}/api/v1/tags`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ name }),
      });
      document.querySelector("#new-tag").value = "";
      refreshTags();
    } catch (e) {
      setStatus(`Add tag failed: ${e}`);
    }
  });

  document.querySelector("#add-folder").addEventListener("click", async () => {
    const name = document.querySelector("#new-folder").value;
    if (!name) return;
    const base = await apiBase();
    try {
      await fetch(`${base}/api/v1/folders`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ name }),
      });
      document.querySelector("#new-folder").value = "";
      refreshTags();
    } catch (e) {
      setStatus(`Add folder failed: ${e}`);
    }
  });

  try {
    document.querySelector("#api-base").textContent = `REST API: ${await apiBase()}/api/v1`;
  } catch {
    document.querySelector("#api-base").textContent = "REST API: unavailable";
  }

  refreshProfiles();
  refreshSessions();
  refreshTags();
});
