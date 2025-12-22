// ===== Configuration =====
const wsUrl = (location.protocol === 'https:' ? 'wss://' : 'ws://') + location.host + '/ws';
const commands = ['/name', '/msg', '/list', '/history', '/join', '/rooms', '/register', '/login', '/help', '/who', '/leave', '/room'];
const REACTION_EMOJIS = ['👍', '❤️', '😂', '😮', '😢', '🎉'];

// Notification sound (short beep as base64)
const NOTIFICATION_SOUND = 'data:audio/wav;base64,UklGRnoGAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQoGAACBhYqFbF1peYBymXyagI+Sf5WRj5GLoZeYmJ2Xq6eTk5CRkZmPk5KRkoyTkpKYjZaUk5WRkJGUlZWOk5WVk5KPkpOVlZCSlZWTko+Sk5WVkJKVlZOSj5KTlZWQkpWVk5KPkpOVlZCSlZWTko+Sk5SVkJKVlZOSj5KTlJWQkpWVk5KPkpOUlZCSlZWTko+Sk5SVkJKVlZOSj5KTlJWQkpWVk5KPkpOUlQ==';
