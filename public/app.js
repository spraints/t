function go() {
  const tmpl = document.querySelector("#content-template")
  const node = tmpl.content.cloneNode(true)

  // todo: fill in fields like in https://developer.mozilla.org/en-US/docs/Web/HTML/Element/template

  document.querySelector("#content").replaceChildren(node)
}

setTimeout(go, 1000)
