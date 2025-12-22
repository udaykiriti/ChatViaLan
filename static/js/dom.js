// ===== DOM Elements =====
const $ = id => document.getElementById(id);

const DOM = {
    statusDot: null,
    connStatus: null,
    roomBadge: null,
    nameInput: null,
    setNameBtn: null,
    messagesEl: null,
    textInput: null,
    sendBtn: null,
    fileInput: null,
    uploadBtn: null,
    uploadMsg: null,
    userList: null,
    userCount: null,
    roomList: null,
    newRoomInput: null,
    joinRoomBtn: null,
    authMsg: null,
    cmdSuggest: null,
    themeToggle: null,
    typingIndicator: null,
    typingText: null,
};

function initDOM() {
    DOM.statusDot = $('statusDot');
    DOM.connStatus = $('connectionStatus');
    DOM.roomBadge = $('currentRoom');
    DOM.nameInput = $('nameInput');
    DOM.setNameBtn = $('setNameBtn');
    DOM.messagesEl = $('messages');
    DOM.textInput = $('textInput');
    DOM.sendBtn = $('sendBtn');
    DOM.fileInput = $('fileInput');
    DOM.uploadBtn = $('uploadBtn');
    DOM.uploadMsg = $('uploadMsg');
    DOM.userList = $('userList');
    DOM.userCount = $('userCount');
    DOM.roomList = $('roomList');
    DOM.newRoomInput = $('newRoomInput');
    DOM.joinRoomBtn = $('joinRoomBtn');
    DOM.authMsg = $('authMsg');
    DOM.cmdSuggest = $('cmdSuggest');
    DOM.themeToggle = $('themeToggle');
    DOM.typingIndicator = $('typingIndicator');
    DOM.typingText = $('typingText');
}
