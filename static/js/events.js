// ===== Input State =====

function updateInputState() {
    const disabled = !connected || !named;
    DOM.textInput.disabled = disabled;
    DOM.sendBtn.disabled = disabled;
    DOM.uploadBtn.disabled = disabled;
    DOM.nameInput.disabled = !connected;
    DOM.setNameBtn.disabled = !connected;
}

// ===== Command Suggestions =====

function showCmdSuggest(items) {
    DOM.cmdSuggest.innerHTML = items.map((c, i) => `<li class="${i === 0 ? 'active' : ''}">${c}</li>`).join('');
    const r = DOM.textInput.getBoundingClientRect();
    DOM.cmdSuggest.style.left = r.left + 'px';
    DOM.cmdSuggest.style.bottom = (window.innerHeight - r.top + 4) + 'px';
    DOM.cmdSuggest.style.width = r.width + 'px';
    DOM.cmdSuggest.style.display = 'block';
}

function hideCmdSuggest() {
    DOM.cmdSuggest.style.display = 'none';
}

// ===== Auth Message =====

function showAuthMsg(msg, isError = false) {
    DOM.authMsg.textContent = msg;
    DOM.authMsg.style.display = 'block';
    DOM.authMsg.style.background = isError ? 'rgba(239,68,68,0.2)' : 'rgba(34,197,94,0.2)';
    DOM.authMsg.style.color = isError ? 'var(--danger)' : 'var(--success)';
    setTimeout(() => DOM.authMsg.style.display = 'none', 3000);
}

// ===== Event Listeners =====

function initEventListeners() {
    // Theme toggle
    DOM.themeToggle.onclick = toggleTheme;

    // Name setting (Enter key support)
    DOM.nameInput.onkeydown = (e) => {
        if (e.key === 'Enter') DOM.setNameBtn.click();
    };

    DOM.setNameBtn.onclick = () => {
        const n = DOM.nameInput.value.trim();
        if (!n) return;
        sendCommand('/name ' + n);
        myName = n;
        named = true;
        updateInputState();
    };

    // Send message
    DOM.sendBtn.onclick = () => {
        const t = DOM.textInput.value.trim();
        if (!t) return;
        t.startsWith('/') ? sendCommand(t) : sendMessage(t);
        DOM.textInput.value = '';
        hideCmdSuggest();
        closeEmojiPicker();
    };

    // Keyboard shortcuts
    DOM.textInput.onkeydown = (e) => {
        // Enter to send
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            DOM.sendBtn.click();
            return;
        }

        // Escape to close things
        if (e.key === 'Escape') {
            hideCmdSuggest();
            closeEmojiPicker();
            return;
        }

        // Command suggestions navigation
        if (DOM.cmdSuggest.style.display === 'block') {
            const items = [...DOM.cmdSuggest.querySelectorAll('li')];
            const active = DOM.cmdSuggest.querySelector('.active');
            let idx = items.indexOf(active);
            if (e.key === 'ArrowDown') { e.preventDefault(); idx = (idx + 1) % items.length; }
            else if (e.key === 'ArrowUp') { e.preventDefault(); idx = (idx - 1 + items.length) % items.length; }
            else if (e.key === 'Tab' && active) { e.preventDefault(); DOM.textInput.value = active.textContent + ' '; hideCmdSuggest(); return; }
            items.forEach(i => i.classList.remove('active'));
            if (items[idx]) items[idx].classList.add('active');
        }
    };

    // Global keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        // Escape to close modals
        if (e.key === 'Escape') {
            hideCmdSuggest();
            closeEmojiPicker();
            hideSearchResults();
            closeSidebar();
        }

        // / to focus input (if not already focused)
        if (e.key === '/' && document.activeElement !== DOM.textInput && document.activeElement.tagName !== 'INPUT') {
            e.preventDefault();
            DOM.textInput.focus();
            DOM.textInput.value = '/';
        }
    });

    // Input handling
    DOM.textInput.oninput = () => {
        const v = DOM.textInput.value.trim();
        if (v.startsWith('/')) {
            const matches = commands.filter(c => c.startsWith(v.toLowerCase()));
            if (matches.length) showCmdSuggest(matches);
            else hideCmdSuggest();
            sendTypingStatus(false);
        } else {
            hideCmdSuggest();
            if (v.length > 0) {
                sendTypingStatus(true);
                clearTimeout(typingTimeout);
                typingTimeout = setTimeout(() => sendTypingStatus(false), 2000);
            } else {
                sendTypingStatus(false);
            }
        }
    };

    // Search
    if (DOM.searchInput) {
        DOM.searchInput.oninput = () => {
            const q = DOM.searchInput.value;
            DOM.clearSearch.classList.toggle('hidden', !q);
            searchMessages(q);
        };

        DOM.searchInput.onkeydown = (e) => {
            if (e.key === 'Escape') {
                DOM.searchInput.value = '';
                hideSearchResults();
            }
        };
    }

    if (DOM.clearSearch) {
        DOM.clearSearch.onclick = () => {
            DOM.searchInput.value = '';
            DOM.clearSearch.classList.add('hidden');
            hideSearchResults();
        };
    }

    if (DOM.closeSearch) {
        DOM.closeSearch.onclick = hideSearchResults;
    }

    // Emoji picker
    if (DOM.emojiBtn) {
        DOM.emojiBtn.onclick = (e) => {
            e.stopPropagation();
            toggleEmojiPicker();
        };
    }

    if (DOM.emojiPicker) {
        DOM.emojiPicker.onclick = (e) => {
            const btn = e.target.closest('.emoji-item');
            if (btn) {
                insertEmoji(btn.dataset.emoji);
            }
        };
    }

    // Room/User clicks
    DOM.roomList.onclick = (e) => {
        const item = e.target.closest('.room-item');
        if (item) {
            sendCommand('/join ' + item.dataset.room);
            closeSidebar();
        }
    };

    DOM.userList.onclick = (e) => {
        // DM button
        const dmBtn = e.target.closest('.dm-btn');
        if (dmBtn) {
            DOM.textInput.value = `/msg ${dmBtn.dataset.user} `;
            DOM.textInput.focus();
            closeSidebar();
            return;
        }

        // User item click for Profile Modal
        const item = e.target.closest('.user-item');
        if (item) {
            showProfileModal(item.dataset.user);
            closeSidebar();
        }
    };

    DOM.joinRoomBtn.onclick = () => {
        const r = DOM.newRoomInput.value.trim();
        if (r) { sendCommand('/join ' + r); DOM.newRoomInput.value = ''; closeSidebar(); }
    };

    DOM.newRoomInput.onkeydown = (e) => {
        if (e.key === 'Enter') DOM.joinRoomBtn.click();
    };

    // Auth tabs
    document.querySelectorAll('.auth-tab').forEach(tab => {
        tab.onclick = () => {
            document.querySelectorAll('.auth-tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.auth-content').forEach(c => c.classList.remove('active'));
            tab.classList.add('active');
            $(tab.dataset.tab + 'Tab').classList.add('active');
        };
    });

    // Login/Register with Enter key
    $('loginUser').onkeydown = (e) => { if (e.key === 'Enter') $('loginPass').focus(); };
    $('loginPass').onkeydown = (e) => { if (e.key === 'Enter') $('loginBtn').click(); };
    $('regUser').onkeydown = (e) => { if (e.key === 'Enter') $('regPass').focus(); };
    $('regPass').onkeydown = (e) => { if (e.key === 'Enter') $('regBtn').click(); };

    $('loginBtn').onclick = () => {
        const u = $('loginUser').value.trim(), p = $('loginPass').value;
        if (!u || !p) return showAuthMsg('Fill all fields', true);

        const btn = $('loginBtn');
        btn.classList.add('loading');
        btn.textContent = 'Authenticating...';

        sendCommand(`/login ${u} ${p}`);

        // Remove loading state after 5 seconds if no response
        setTimeout(() => {
            btn.classList.remove('loading');
            btn.textContent = 'Login';
        }, 5000);
    };

    $('regBtn').onclick = () => {
        const u = $('regUser').value.trim(), p = $('regPass').value;
        if (!u || !p) return showAuthMsg('Fill all fields', true);

        const btn = $('regBtn');
        btn.classList.add('loading');
        btn.textContent = 'Processing...';

        sendCommand(`/register ${u} ${p}`);

        // Remove loading state after 5 seconds if no response
        setTimeout(() => {
            btn.classList.remove('loading');
            btn.textContent = 'Register';
        }, 5000);
    };

    // File upload with progress
    DOM.uploadBtn.onclick = () => {
        if (!DOM.fileInput.files?.length) return DOM.uploadMsg.textContent = 'Select a file first';

        const file = DOM.fileInput.files[0];
        const fd = new FormData();
        fd.append('file', file);

        DOM.uploadBtn.disabled = true;
        DOM.uploadMsg.textContent = 'Uploading...';

        // Show progress bar
        const progressEl = document.getElementById('uploadProgress');
        if (progressEl) {
            progressEl.style.display = 'block';
            progressEl.value = 0;
            progressEl.max = 100;
        }

        const xhr = new XMLHttpRequest();
        xhr.open('POST', '/upload', true);

        xhr.upload.onprogress = (e) => {
            if (e.lengthComputable) {
                const percent = (e.loaded / e.total) * 100;
                if (progressEl) progressEl.value = percent;
                DOM.uploadMsg.textContent = `${Math.round(percent)}%`;
            }
        };

        xhr.onload = () => {
            if (xhr.status === 200) {
                try {
                    const j = JSON.parse(xhr.responseText);
                    if (j.ok && j.files?.length) {
                        j.files.forEach(f => sendMessage(`📎 ${f.filename}: ${location.origin}${f.url}`));
                        DOM.uploadMsg.textContent = 'Done!';
                        DOM.fileInput.value = '';
                    } else {
                        DOM.uploadMsg.textContent = 'Upload failed';
                    }
                } catch (e) {
                    DOM.uploadMsg.textContent = 'Error parsing response';
                }
            } else {
                DOM.uploadMsg.textContent = `Error: ${xhr.status}`;
            }
            DOM.uploadBtn.disabled = false;
            // Hide progress after success
            setTimeout(() => {
                if (progressEl) {
                    progressEl.style.display = 'none';
                    progressEl.value = 0;
                }
                DOM.uploadMsg.textContent = '';
            }, 3000);
        };

        xhr.onerror = () => {
            DOM.uploadMsg.textContent = 'Network Error';
            DOM.uploadBtn.disabled = false;
        };

        xhr.send(fd);
    };

    // Close suggestions / emoji picker on outside click
    document.onclick = (e) => {
        if (!DOM.cmdSuggest.contains(e.target) && e.target !== DOM.textInput) hideCmdSuggest();

        if (!e.target.closest('.add-reaction-btn') && !e.target.closest('.reaction-picker')) {
            document.querySelectorAll('.reaction-picker').forEach(p => p.remove());
        }

        if (!e.target.closest('.emoji-picker') && !e.target.closest('.emoji-btn')) {
            closeEmojiPicker();
        }

        // Handle Reaction Picker Clicks (Global because it's in body)
        const pickerEmoji = e.target.closest('.picker-emoji');
        if (pickerEmoji) {
            sendReaction(pickerEmoji.dataset.msgId, pickerEmoji.dataset.emoji);
            document.querySelectorAll('.reaction-picker').forEach(p => p.remove());
        }
    };

    DOM.cmdSuggest.onclick = (e) => {
        if (e.target.tagName === 'LI') {
            DOM.textInput.value = e.target.textContent + ' ';
            hideCmdSuggest();
            DOM.textInput.focus();
        }
    };

    // Global click delegations for dynamic content (Search, Pins, Links)
    document.addEventListener('click', (e) => {
        const searchItem = e.target.closest('.search-item');
        if (searchItem) {
            jumpToMessage(searchItem.dataset.msgId);
            hideSearchResults();
        }

        const pinnedItem = e.target.closest('.pinned-item');
        if (pinnedItem) {
            jumpToMessage(pinnedItem.dataset.msgId);
        }

        const replyPreview = e.target.closest('.reply-preview');
        if (replyPreview && replyPreview.onclick) {
            // Already handled by inline onclick, but for robustness:
            // jumpToMessage is global, so inline works. 
            // We leave inline alone or move it here. 
            // Current impl uses inline, so no action needed for Reply Preview.
        }
    });

    // Message area clicks (reactions, edit, delete)
    DOM.messagesEl.onclick = (e) => {
        const reactionBtn = e.target.closest('.reaction-btn');
        if (reactionBtn) {
            sendReaction(reactionBtn.dataset.msgId, reactionBtn.dataset.emoji);
            return;
        }

        const addReactionBtn = e.target.closest('.add-reaction-btn');
        if (addReactionBtn) {
            document.querySelectorAll('.reaction-picker').forEach(p => p.remove());
            const rect = addReactionBtn.getBoundingClientRect();
            const pickerHtml = createReactionPicker(addReactionBtn.dataset.msgId);
            document.body.insertAdjacentHTML('beforeend', pickerHtml);
            const picker = document.querySelector('.reaction-picker');
            if (picker) {
                picker.style.top = (rect.bottom + 4) + 'px';
                picker.style.left = rect.left + 'px';
            }
            return;
        }


        const editBtn = e.target.closest('.edit-btn');
        if (editBtn) {
            const msgEl = document.querySelector(`[data-msg-id="${editBtn.dataset.msgId}"]`);
            const bubble = msgEl.querySelector('.message-bubble');
            if (bubble) {
                const newText = prompt('Edit message:', bubble.textContent);
                if (newText && newText !== bubble.textContent) {
                    sendEdit(editBtn.dataset.msgId, newText);
                }
            }
            return;
        }

        const deleteBtn = e.target.closest('.delete-btn');
        if (deleteBtn && confirm('Delete this message?')) {
            sendDelete(deleteBtn.dataset.msgId);
        }

        const replyBtn = e.target.closest('.reply-btn');
        if (replyBtn) {
            const msgId = replyBtn.dataset.msgId;
            const msg = allMessages.find(m => m.id === msgId);
            if (msg) {
                DOM.textInput.placeholder = `Replying to ${msg.from}...`;
                DOM.textInput.dataset.replyTo = msgId;
                DOM.textInput.focus();
            }
        }

        const pinBtn = e.target.closest('.pin-btn');
        if (pinBtn) {
            sendCommand(`/pin ${pinBtn.dataset.msgId}`);
        }
    };

    // Window focus/blur
    window.onfocus = () => {
        windowFocused = true;
        clearUnread();
        if (lastMsgId && connected) {
            ws.send(JSON.stringify({ type: 'markread', last_msg_id: lastMsgId }));
        }
    };
    window.onblur = () => { windowFocused = false; };

    // Modal close events
    DOM.modalCloseBtn.onclick = hideProfileModal;
    DOM.profileModalOverlay.onclick = (e) => {
        if (e.target === DOM.profileModalOverlay) hideProfileModal();
    };

    // Markdown Help Modal Events
    if (DOM.mdHelpBtn) {
        DOM.mdHelpBtn.onclick = () => {
            DOM.mdHelpOverlay.classList.add('active');
        };
    }
    if (DOM.mdHelpCloseBtn) {
        DOM.mdHelpCloseBtn.onclick = () => {
            DOM.mdHelpOverlay.classList.remove('active');
        };
    }
    if (DOM.mdHelpOverlay) {
        DOM.mdHelpOverlay.onclick = (e) => {
            if (e.target === DOM.mdHelpOverlay) DOM.mdHelpOverlay.classList.remove('active');
        };
    }
}

// ===== Profile Modal Functions =====

function showProfileModal(user) {
    const status = userStatuses[user] || 'active';
    DOM.modalUserName.textContent = user;
    DOM.modalAvatar.textContent = user[0].toUpperCase();
    DOM.modalUserStatus.textContent = status.charAt(0).toUpperCase() + status.slice(1);
    DOM.modalUserStatus.className = `status-badge ${status}`;

    DOM.modalMessageBtn.onclick = () => {
        DOM.textInput.value = `/msg ${user} `;
        DOM.textInput.focus();
        hideProfileModal();
    };

    DOM.modalMentionBtn.onclick = () => {
        DOM.textInput.value = '@' + user + ' ';
        DOM.textInput.focus();
        hideProfileModal();
    };

    DOM.profileModalOverlay.classList.add('active');
}

function hideProfileModal() {
    DOM.profileModalOverlay.classList.remove('active');
}
