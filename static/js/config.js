// ===== Configuration =====
const wsUrl = (location.protocol === 'https:' ? 'wss://' : 'ws://') + location.host + '/ws';
const commands = ['/name', '/msg', '/list', '/history', '/join', '/rooms', '/register', '/login', '/help', '/who', '/leave', '/room', '/pin', '/unpin'];
const REACTION_EMOJIS = ['👍', '❤️', '😂', '😮', '😢', '🎉'];

// Full emoji list for picker
const EMOJI_CATEGORIES = {
    'Smileys': ['😀', '😁', '😂', '🤣', '😃', '😄', '😅', '😆', '😉', '😊', '😋', '😎', '😍', '🥰', '😘', '😗', '😙', '😚', '🥲', '🤗', '🤔', '🫡', '🤫', '🤭', '🫢', '🫣', '🤥', '😌', '😔', '😪', '😴', '🤤', '😷', '🤒', '🤕', '🤢', '🤮', '🤧', '🥵', '🥶', '🥴', '😵', '🤯', '🤠', '🥳', '🥸', '😎', '🤓', '🧐'],
    'Gestures': ['👍', '👎', '👊', '✊', '🤛', '🤜', '🤝', '👏', '🙌', '👐', '🤲', '🤞', '✌️', '🤟', '🤘', '👌', '🤌', '🤏', '👈', '👉', '👆', '👇', '☝️', '✋', '🤚', '🖐️', '🖖', '👋', '🤙', '💪', '🦾', '🖕', '✍️', '🙏'],
    'Hearts': ['❤️', '🧡', '💛', '💚', '💙', '💜', '🖤', '🤍', '🤎', '💔', '❤️‍🔥', '❤️‍🩹', '💕', '💞', '💓', '💗', '💖', '💘', '💝', '💟'],
    'Objects': ['📎', '📁', '📂', '📄', '📃', '📑', '🔗', '💻', '⌨️', '🖥️', '📱', '📷', '🎥', '🎬', '🎤', '🎧', '🎵', '🎶', '🔔', '📢', '💡', '🔒', '🔓', '🔑', '🗝️', '⚙️', '🛠️', '📌', '📍'],
    'Symbols': ['✅', '❌', '⭐', '🌟', '💫', '⚡', '🔥', '💥', '❗', '❓', '‼️', '⁉️', '💯', '🎯', '✨', '🌈', '☀️', '🌙', '⭕', '❎']
};

// Notification sound
const NOTIFICATION_SOUND = 'data:audio/wav;base64,UklGRnoGAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQoGAACBhYqFbF1peYBymXyagI+Sf5WRj5GLoZeYmJ2Xq6eTk5CRkZmPk5KRkoyTkpKYjZaUk5WRkJGUlZWOk5WVk5KPkpOVlZCSlZWTko+Sk5WVkJKVlZOSj5KTlZWQkpWVk5KPkpOVlZCSlZWTko+Sk5SVkJKVlZOSj5KTlJWQkpWVk5KPkpOUlZCSlZWTko+Sk5SVkJKVlZOSj5KTlJWQkpWVk5KPkpOUlQ==';

// Image extensions for preview
const IMAGE_EXTENSIONS = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'bmp'];
