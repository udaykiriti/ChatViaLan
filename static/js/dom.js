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
    menuBtn: null,
    sidebar: null,
    sidebarOverlay: null,
    closeSidebar: null,
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
    DOM.menuBtn = $('menuBtn');
    DOM.sidebar = $('sidebar');
    DOM.sidebarOverlay = $('sidebarOverlay');
    DOM.closeSidebar = $('closeSidebar');
}

// ===== Mobile Sidebar Toggle =====
function openSidebar() {
    if (DOM.sidebar) DOM.sidebar.classList.add('open');
    if (DOM.sidebarOverlay) DOM.sidebarOverlay.classList.add('open');
    document.body.style.overflow = 'hidden';
}

function closeSidebar() {
    if (DOM.sidebar) DOM.sidebar.classList.remove('open');
    if (DOM.sidebarOverlay) DOM.sidebarOverlay.classList.remove('open');
    document.body.style.overflow = '';
}

function initMobileMenu() {
    if (DOM.menuBtn) {
        DOM.menuBtn.onclick = openSidebar;
    }
    if (DOM.sidebarOverlay) {
        DOM.sidebarOverlay.onclick = closeSidebar;
    }
    if (DOM.closeSidebar) {
        DOM.closeSidebar.onclick = closeSidebar;
    }
}
