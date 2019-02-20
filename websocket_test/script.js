var wsUri = "ws://127.0.0.1:8080";
var output;
var websocketConn = null;

var connectButton = null;
var disconnectButton = null;
var submitMessageButton = null
var clearMessagesButton = null;
var messagesTextArea = null;
var websockerUrl = null;
var messageToSend = null;

function init() {
    console.log("Init");
    connectButton = $('#websocket_connect');
    disconnectButton = $('#websocket_disconnect');
    submitMessageButton = $('#submit_message');
    clearMessagesButton = $('#clear_messages');
    messagesTextArea = $('#messages');
    websockerUrl = $('#websocket_url');
    messageToSend = $('#submit_message_textbox');

    connectButton.click(function() {
        console.log("Connect button clicked");
        var addr = websockerUrl.val();
        createWebsocket(addr);
    });

    submitMessageButton.click(function(){
        var msg = messagesTextArea.val();
        if (websocketConn !== null) {
            websocket.send(message);
            messagesTextArea.val('');
        }
    });

    clearMessagesButton.click(function(){
        messagesTextArea.val('');
    });

}

function createWebsocket(addr) {
    if (websocketConn === null) {
        console.log("Opening new websocket connection");
        websocket = new WebSocket(addr);
        websocket.onopen = function (evt) { onOpen(evt) };
        websocket.onclose = function (evt) { onClose(evt) };
        websocket.onmessage = function (evt) { onMessage(evt) };
        websocket.onerror = function (evt) { onError(evt) };
    } else {
        console.log("Websocket connection already exists, not creating new one");
    }
}

function onOpen(evt) {
    writeMessage("CONNECTED");
}

function onClose(evt) {
    writeMessage("DISCONNECTED");
}

function onMessage(evt) {
    writeMessage(evt.data);
    websocket.close();
}

function onError(evt) {
    writeMessage(evt.data);
}

function writeMessage(message) {
    var date = Date.now();
    var currentMessages = messagesTextArea.val();
    var newMsg = `${currentMessages}\n${date} : ${message}`;
    messagesTextArea.val(newMsg);
}

window.addEventListener("load", init, false);