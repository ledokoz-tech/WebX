// Initialization script for the webview
// This script runs when the webview is created

console.log('🌐 WebX Browser initialized');
console.log('Version 0.1.0');
console.log('Official system browser for Ledokoz OS');

// Setup IPC communication with the Rust backend
window.ipc = {
    send: function (message) {
        window.ipc.postMessage(JSON.stringify(message));
    }
};

// Intercept navigation events
window.addEventListener('beforeunload', function (e) {
    window.ipc.send({
        type: 'beforeunload',
        url: window.location.href
    });
});

// Track page load
window.addEventListener('load', function () {
    window.ipc.send({
        type: 'pageload',
        url: window.location.href,
        title: document.title
    });
});

// Track title changes
const titleObserver = new MutationObserver(function (mutations) {
    window.ipc.send({
        type: 'titlechange',
        title: document.title
    });
});

titleObserver.observe(
    document.querySelector('title') || document.querySelector('head'),
    { subtree: true, characterData: true, childList: true }
);

// Keyboard shortcuts
document.addEventListener('keydown', function (e) {
    // Ctrl/Cmd + T: New tab
    if ((e.ctrlKey || e.metaKey) && e.key === 't') {
        e.preventDefault();
        window.ipc.send({ type: 'newtab' });
    }

    // Ctrl/Cmd + W: Close tab
    if ((e.ctrlKey || e.metaKey) && e.key === 'w') {
        e.preventDefault();
        window.ipc.send({ type: 'closetab' });
    }

    // Ctrl/Cmd + R: Refresh
    if ((e.ctrlKey || e.metaKey) && e.key === 'r') {
        e.preventDefault();
        window.ipc.send({ type: 'refresh' });
    }

    // Ctrl/Cmd + L: Focus address bar
    if ((e.ctrlKey || e.metaKey) && e.key === 'l') {
        e.preventDefault();
        window.ipc.send({ type: 'focusaddress' });
    }

    // Ctrl/Cmd + D: Bookmark
    if ((e.ctrlKey || e.metaKey) && e.key === 'd') {
        e.preventDefault();
        window.ipc.send({
            type: 'bookmark',
            url: window.location.href,
            title: document.title
        });
    }

    // Alt + Left: Back
    if (e.altKey && e.key === 'ArrowLeft') {
        e.preventDefault();
        window.ipc.send({ type: 'back' });
    }

    // Alt + Right: Forward
    if (e.altKey && e.key === 'ArrowRight') {
        e.preventDefault();
        window.ipc.send({ type: 'forward' });
    }
});

// Context menu customization
document.addEventListener('contextmenu', function (e) {
    const target = e.target;
    const contextData = {
        type: 'contextmenu',
        x: e.clientX,
        y: e.clientY,
        hasLink: target.tagName === 'A',
        hasImage: target.tagName === 'IMG',
        hasSelection: window.getSelection().toString().length > 0,
        link: target.tagName === 'A' ? target.href : null,
        image: target.tagName === 'IMG' ? target.src : null,
        selection: window.getSelection().toString()
    };

    window.ipc.send(contextData);
});

console.log('✅ WebX initialization complete');
