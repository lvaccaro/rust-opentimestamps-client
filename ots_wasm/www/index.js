import * as ots from "ots_wasm"

const infoButton = document.getElementById("info-button")
const stampButton = document.getElementById("stamp-button")
const otsText = document.getElementById("ots-text")
const digestText = document.getElementById("digest-text")

infoButton.disabled = false  // The button start disabled and it's enabled here once the wasm has been loaded
infoButton.addEventListener("click", infoButtonPressed)
stampButton.disabled = false  // The button start disabled and it's enabled here once the wasm has been loaded
stampButton.addEventListener("click", stampButtonPressed)

async function stampButtonPressed(e) {
    try {
        info.innerText = "Loading..."
        stampButton.disabled = true
        let infoString = await ots.stamp(digestText.value.toString())
        console.log(infoString)
        info.innerText = infoString
    } catch (e) {
        console.log("error")
        info.innerText = e
    } finally {
        console.log("completed")
        stampButton.disabled = false
    }
}
async function infoButtonPressed(e) {
    try {
        info.innerText = "Loading..."
        infoButton.disabled = true
        let infoString = ots.info(otsText.value.toString())
        console.log(infoString)
        info.innerText = infoString
    } catch (e) {
        console.log("error")
        info.innerText = e
    } finally {
        console.log("completed")
        infoButton.disabled = false
    }
}
