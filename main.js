const {app, Tray, Menu} = require('electron')
const path = require('path')
const Promise = require('promise')
const exec = Promise.denodeify(require('child_process').exec)

const POLL_INTERVAL = 5 * 1000

const assetsDir = path.join(__dirname, 'assets')
const whiteCircle = path.join(assetsDir, 'white-circle.png')
const blackCircle = path.join(assetsDir, 'black-circle.png')
const blueCircle = path.join(assetsDir, 'blue-circle.png')

let tray = undefined

app.dock.hide()

app.on('ready', () => {
  createTray()
  updateIcon()
  setInterval(updateIcon, POLL_INTERVAL)
})

const createTray = () => {
  let menu = Menu.buildFromTemplate([
      {label: 'Quit', click: () => app.quit()},
  ])
  tray = new Tray(blackCircle)
  tray.setContextMenu(menu)
}

const updateIcon = () => {
  getState().then(setIcon)
}

const getState = () => {
  return exec("t status").then((stdout, stderr) => {
    if (stdout == "WORKING\n") {
      return "working"
    } else if (stdout == "NOT working\n") {
      return "not-working"
    } else {
      return "unknown"
    }
  })
}

const setIcon = (state) => {
  switch (state) {
    case 'working':
      tray.setImage(blueCircle)
      tray.setTooltip('WORKING')
      break
    case 'not-working':
      tray.setImage(whiteCircle)
      tray.setTooltip('NOT working')
      break
    default:
      tray.setImage(blackCircle)
      tray.setTooltip('unknown')
      break
  }
}
