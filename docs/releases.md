# Downloads

Get the latest RustyRunways desktop builds for your platform. These links point to the most recent GitHub Release assets.

- Latest release page: https://github.com/DennisLent/RustyRunways/releases/latest
- All releases: https://github.com/DennisLent/RustyRunways/releases

## Direct downloads

The links below are populated automatically from the latest release assets.

- macOS (Apple Silicon): <a id="dl-mac-arm" href="https://github.com/DennisLent/RustyRunways/releases/latest">Loading…</a>
- macOS (Intel): <a id="dl-mac-intel" href="https://github.com/DennisLent/RustyRunways/releases/latest">Loading…</a>
- macOS (Universal): <a id="dl-mac-universal" href="https://github.com/DennisLent/RustyRunways/releases/latest">Loading…</a>
- Windows: <a id="dl-windows" href="https://github.com/DennisLent/RustyRunways/releases/latest">Loading…</a>
- Linux AppImage: <a id="dl-linux-appimage" href="https://github.com/DennisLent/RustyRunways/releases/latest">Loading…</a>
- Linux Debian package: <a id="dl-linux-deb" href="https://github.com/DennisLent/RustyRunways/releases/latest">Loading…</a>

<script>
(function(){
  const owner = 'DennisLent';
  const repo = 'RustyRunways';
  const latestApi = `https://api.github.com/repos/${owner}/${repo}/releases/latest`;
  function setLink(id, asset){
    const el = document.getElementById(id);
    if(!el) return;
    if(asset){
      el.textContent = asset.name + ` (${(asset.size/1024/1024).toFixed(1)} MB)`;
      el.href = asset.browser_download_url;
    } else {
      el.textContent = 'Not available in latest release';
    }
  }
  function find(assets, fn){ return assets.find(fn); }
  function has(name, parts){ return parts.every(p => name.toLowerCase().includes(p)); }
  fetch(latestApi).then(r => r.json()).then(rel => {
    const assets = Array.isArray(rel.assets) ? rel.assets : [];
    // macOS
    const macDmg = assets.filter(a => a.name.endsWith('.dmg'));
    const macArm = find(macDmg, a => has(a.name, ['arm64']) || has(a.name, ['aarch64']));
    const macIntel = find(macDmg, a => has(a.name, ['x86_64']) || has(a.name, ['x64']));
    const macUniversal = find(macDmg, a => has(a.name, ['universal']));
    setLink('dl-mac-arm', macArm || macUniversal);
    setLink('dl-mac-intel', macIntel || macUniversal);
    setLink('dl-mac-universal', macUniversal);
    // Windows
    const win = find(assets, a => a.name.endsWith('.msi') || a.name.endsWith('.exe'));
    setLink('dl-windows', win);
    // Linux
    const appimage = find(assets, a => a.name.toLowerCase().endsWith('.appimage'));
    const deb = find(assets, a => a.name.toLowerCase().endsWith('.deb'));
    setLink('dl-linux-appimage', appimage);
    setLink('dl-linux-deb', deb);
  }).catch(() => {
    // Leave default links to latest page
  });
})();
</script>

## Install notes

- macOS: You may need to allow the app in System Settings if your build isn’t notarized.
- Windows: The .msi installer will guide you through installation (unsigned builds may show SmartScreen).
- Linux: Prefer the AppImage for a portable run or the .deb for Debian/Ubuntu.

