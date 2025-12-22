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

    // Name setting
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
    };

    // Keyboard handling
    DOM.textInput.onkeydown = (e) => {
        if (e.key === 'Enter') { DOM.sendBtn.click(); return; }
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

    // Room/User clicks
    DOM.roomList.onclick = (e) => {
        const item = e.target.closest('.room-item');
        if (item) sendCommand('/join ' + item.dataset.room);
    };

    DOM.userList.onclick = (e) => {
        const item = e.target.closest('.user-item');
        if (item) {
            DOM.textInput.value = '@' + item.dataset.user + ' ';
            DOM.textInput.focus();
        }
    };

    DOM.joinRoomBtn.onclick = () => {
        const r = DOM.newRoomInput.value.trim();
        if (r) { sendCommand('/join ' + r); DOM.newRoomInput.value = ''; }
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

    document.querySelectorAll('.switch-tab').forEach(btn => {
        btn.onclick = () => document.querySelector(`.auth-tab[data-target="${btn.dataset.target}"]`).click();
    });

    // Login/Register
    $('loginBtn').onclick = () => {
        const u = $('loginUser').value.trim(), p = $('loginPass').value;
        if (!u || !p) return showAuthMsg('Fill all fields', true);
        sendCommand(`/login ${u} ${p}`);
    };

    $('regBtn').onclick = () => {
        const u = $('regUser').value.trim(), p = $('regPass').value;
        if (!u || !p) return showAuthMsg('Fill all fields', true);
        sendCommand(`/register ${u} ${p}`);
    };

    // File upload
    DOM.uploadBtn.onclick = async () => {
        if (!DOM.fileInput.files?.length) return DOM.uploadMsg.textContent = 'Select a file first';
        const fd = new FormData();
        fd.append('file', DOM.fileInput.files[0]);
        DOM.uploadBtn.disabled = true;
        DOM.uploadMsg.textContent = 'Uploading...';
        try {
            const res = await fetch('/upload', { method: 'POST', body: fd });
            const j = await res.json();
            if (j.ok && j.files?.length) {
                j.files.forEach(f => sendMessage(`📎 ${f.filename}: ${location.origin}${f.url}`));
                DOM.uploadMsg.textContent = 'Done!';
                DOM.fileInput.value = '';
            } else DOM.uploadMsg.textContent = 'Upload failed';
        } catch (e) { DOM.uploadMsg.textContent = 'Error: ' + e; }
        DOM.uploadBtn.disabled = false;
        setTimeout(() => DOM.uploadMsg.textContent = '', 3000);
    };

    // Close suggestions
    document.onclick = (e) => {
        if (!DOM.cmdSuggest.contains(e.target) && e.target !== DOM.textInput) hideCmdSuggest();
        if (!e.target.closest('.add-reaction-btn') && !e.target.closest('.reaction-picker')) {
            document.querySelectorAll('.reaction-picker').forEach(p => p.remove());
        }
    };

    document.onkeydown = (e) => { if (e.key === 'Escape') hideCmdSuggest(); };

    DOM.cmdSuggest.onclick = (e) => {
        if (e.target.tagName === 'LI') {
            DOM.textInput.value = e.target.textContent + ' ';
            hideCmdSuggest();
            DOM.textInput.focus();
        }
    };

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
            addReactionBtn.insertAdjacentHTML('afterend', createReactionPicker(addReactionBtn.dataset.msgId));
            return;
        }

        const pickerEmoji = e.target.closest('.picker-emoji');
        if (pickerEmoji) {
            sendReaction(pickerEmoji.dataset.msgId, pickerEmoji.dataset.emoji);
            document.querySelectorAll('.reaction-picker').forEach(p => p.remove());
            return;
        }

        const editBtn = e.target.closest('.edit-btn');
        if (editBtn) {
            const msgEl = document.querySelector(`[data-msg-id="${editBtn.dataset.msgId}"]`);
            const content = msgEl.querySelector('.message-content');
            const newText = prompt('Edit message:', content.textContent);
            if (newText && newText !== content.textContent) {
                sendEdit(editBtn.dataset.msgId, newText);
            }
            return;
        }

        const deleteBtn = e.target.closest('.delete-btn');
        if (deleteBtn && confirm('Delete this message?')) {
            sendDelete(deleteBtn.dataset.msgId);
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
}
