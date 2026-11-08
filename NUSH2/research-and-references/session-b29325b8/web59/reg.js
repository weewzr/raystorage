(async () => {
  for (let i = 0; i < 200; i++) {
    if (document.readyState === "complete") break;
    await new Promise(r => setTimeout(r, 100));
  }
  const m = await import(location.origin + "/app.js");
  for (let i = 0; i < 300; i++) {
    if (m.state.registry && m.state.result) break;
    await new Promise(r => setTimeout(r, 100));
  }
  return m.state.registry;
})()
