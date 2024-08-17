async function go() {
  const response = await fetch("/api/status")
  if (!response.ok) {
    document.querySelector("#content").innerText = "error loading current status"
    return
  }
  const json = await response.json()

  const tmpl = document.querySelector("#content-template")
  const node = tmpl.content.cloneNode(true)

  node.querySelector(".is-working").textContent = json.working ? "WORKING" : "NOT WORKING"
  node.querySelector(".minutes-this-week").textContent = json.minutes_this_week
  node.querySelector(".last-update").textContent = json.last_update

  document.querySelector("#content").replaceChildren(node)
}

setTimeout(go, 1)
