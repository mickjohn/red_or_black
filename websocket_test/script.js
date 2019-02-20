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

    disconnectButton.click(function() {
        console.log("Closing websocket");
        websocketConn.close();
        websocketConn = null;
    });

    submitMessageButton.click(function(){
        if (websocketConn !== null) {
          var msg = messageToSend.val();
            websocketConn.send(msg);
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
        websocketConn = new WebSocket(addr);
        websocketConn.onopen = function (evt) { onOpen(evt) };
        websocketConn.onclose = function (evt) { onClose(evt) };
        websocketConn.onmessage = function (evt) { onMessage(evt) };
        websocketConn.onerror = function (evt) { onError(evt) };
    } else {
        console.log("Websocket connection already exists, not creating new one");
        writeMessage("websocket connection already open, disconnect first");
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
