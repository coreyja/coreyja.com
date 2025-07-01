// Scroll restoration for view transitions
window.addEventListener('pageswap', (event) => {
    sessionStorage.setItem('scrollPosition', window.scrollY.toString());
});

window.addEventListener('pagereveal', (event) => {
    const savedPosition = sessionStorage.getItem('scrollPosition');
    if (savedPosition && navigation?.activation?.from && navigation?.activation?.entry) {
        const fromURL = new URL(navigation.activation.from.url);
        const currentURL = new URL(navigation.activation.entry.url);
        if (fromURL.pathname === currentURL.pathname) {
            window.scrollTo(0, parseInt(savedPosition));
        }
    }
});