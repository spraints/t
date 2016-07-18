const {app, Tray} = require('electron')
const path = require('path')

const assetsDir = path.join(__dirname, 'assets')
const whiteCircle = path.join(assetsDir, 'white-circle.png')
const blackCircle = path.join(assetsDir, 'black-circle.png')
const blueCircle = path.join(assetsDir, 'blue-circle.png')

let tray = undefined

app.dock.hide()

app.on('ready', () => {
  createTray()
})

const createTray = () => {
  tray = new Tray(blackCircle)
  //tray.on('click', showMenu)
}
