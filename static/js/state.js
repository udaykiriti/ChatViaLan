// ===== Application State =====
let ws = null;
let connected = false;
let named = false;
let currentRoom = 'lobby';
let typingTimeout = null;
let isTyping = false;
let unreadCount = 0;
let windowFocused = true;
let myName = '';
let lastMsgId = null;
let notificationAudio = null;
let soundEnabled = true;
let allMessages = []; // Store all messages for search
let pinnedMessages = []; // Pinned message IDs
let emojiPickerOpen = false;
