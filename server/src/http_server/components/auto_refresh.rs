use maud::{html, Markup, PreEscaped};

/// A reusable auto-refresh button component with visual countdown
///
/// # Example
/// ```
/// // Basic usage - refreshes #my-content every 10 seconds
/// AutoRefreshButton::new("#my-content", "/api/data").render()
///
/// // With custom interval - refreshes every 30 seconds  
/// AutoRefreshButton::new("#my-table", "/api/table-data")
///     .with_interval(30)
///     .render()
/// ```
pub struct AutoRefreshButton {
    /// CSS selector for the element to refresh
    pub target_selector: String,
    /// Interval in seconds between refreshes
    pub interval_seconds: u32,
    /// URL to fetch for refresh content
    pub fetch_url: Option<String>,
}

impl Default for AutoRefreshButton {
    fn default() -> Self {
        Self {
            target_selector: "#content".to_string(),
            interval_seconds: 10,
            fetch_url: None,
        }
    }
}

impl AutoRefreshButton {
    pub fn new(target_selector: impl Into<String>, fetch_url: Option<impl Into<String>>) -> Self {
        Self {
            target_selector: target_selector.into(),
            interval_seconds: 10,
            fetch_url: fetch_url.map(Into::into),
        }
    }

    pub fn with_interval(mut self, seconds: u32) -> Self {
        self.interval_seconds = seconds;
        self
    }

    #[allow(clippy::too_many_lines)]
    pub fn render(&self) -> Markup {
        html! {
            div class="flex flex-col items-center auto-refresh-container"
                data-target=(self.target_selector)
                data-url=[self.fetch_url.as_ref()]
                data-interval=(self.interval_seconds) {
                button class="auto-refresh-button text-gray-600 hover:text-gray-800 transition-colors"
                    aria-label=(format!("Auto-refresh content every {} seconds. Click to refresh now.", self.interval_seconds)) {
                    i class="fas fa-sync-alt text-lg auto-refresh-icon" aria-hidden="true" {}
                }
                div class="w-12 h-1 bg-gray-200 rounded-full mt-2 overflow-hidden" {
                    div class="auto-refresh-progress h-full bg-gray-400 rounded-full transition-none" style="width: 100%;" {}
                }
            }

            style {
                (PreEscaped(format!(r"
                    @keyframes spin-slow {{
                        from {{ transform: rotate(0deg); }}
                        to {{ transform: rotate(360deg); }}
                    }}
                    
                    .spinning {{
                        animation: spin-slow {}s linear infinite;
                    }}
                    
                    @keyframes pulse-spin {{
                        0%, 100% {{ 
                            transform: rotate(0deg) scale(1);
                            opacity: 1;
                        }}
                        50% {{ 
                            transform: rotate(180deg) scale(1.1);
                            opacity: 0.8;
                        }}
                    }}
                    
                    .refreshing {{
                        animation: pulse-spin 0.8s ease-in-out;
                    }}
                    
                    @keyframes deplete {{
                        from {{ width: 100%; }}
                        to {{ width: 0%; }}
                    }}
                    
                    .depleting {{
                        animation: deplete {}s linear;
                    }}
                    
                    @keyframes flash {{
                        0%, 100% {{ opacity: 1; }}
                        50% {{ opacity: 0.6; }}
                    }}
                    
                    .progress-flash {{
                        animation: flash 0.8s ease-in-out;
                    }}
                ", self.interval_seconds, self.interval_seconds)))
            }

            script {
                (PreEscaped(r"
                    // Initialize all auto-refresh containers on the page
                    document.querySelectorAll('.auto-refresh-container').forEach(container => {
                        const refreshButton = container.querySelector('.auto-refresh-button');
                        const refreshIcon = container.querySelector('.auto-refresh-icon');
                        const progressBar = container.querySelector('.auto-refresh-progress');
                        
                        // Read configuration from data attributes
                        const targetSelector = container.dataset.target;
                        const fetchUrl = container.dataset.url || window.location.href;
                        const REFRESH_INTERVAL_SECONDS = parseInt(container.dataset.interval);
                        
                        let refreshInterval;
                        let countdownInterval;
                        let cycleStartTime = Date.now();
                        
                        let isRefreshing = false;
                        
                        // Function to restart animations
                        function restartAnimations() {{
                            // Restart icon spinning animation
                            refreshIcon.classList.remove('spinning');
                            void refreshIcon.offsetWidth; // Force reflow
                            refreshIcon.classList.add('spinning');
                            
                            // Restart progress bar animation
                            progressBar.classList.remove('depleting');
                            progressBar.style.width = '100%';
                            void progressBar.offsetWidth; // Force reflow
                            progressBar.classList.add('depleting');
                        }}
                        
                        
                        // Function to refresh the content
                        async function refreshContent() {{
                            if (isRefreshing) return;
                            isRefreshing = true;
                            
                            // Add refreshing animations
                            refreshIcon.classList.add('refreshing');
                            progressBar.classList.add('progress-flash');
                            
                            try {{
                                const response = await fetch(fetchUrl);
                                const html = await response.text();
                                
                                const parser = new DOMParser();
                                const doc = parser.parseFromString(html, 'text/html');
                                const newContent = doc.querySelector(targetSelector);
                                
                                if (!newContent) return;
                                
                                const currentContent = document.querySelector(targetSelector);
                                if (currentContent && currentContent.parentNode) {{
                                    currentContent.parentNode.replaceChild(newContent.cloneNode(true), currentContent);
                                }}
                            }} catch (error) {{
                                console.error('Failed to refresh content:', error);
                            }} finally {{
                                // Remove refreshing animations after a delay
                                setTimeout(() => {{
                                    refreshIcon.classList.remove('refreshing');
                                    progressBar.classList.remove('progress-flash');
                                    isRefreshing = false;
                                }}, 800);
                            }}
                        }}
                        
                        // Function to start the refresh cycle
                        function startRefreshCycle() {{
                            // Reset cycle start time
                            cycleStartTime = Date.now();
                            
                            // Start animations
                            restartAnimations();
                            
                            
                            // Clear any existing intervals
                            if (refreshInterval) clearInterval(refreshInterval);
                            if (countdownInterval) clearInterval(countdownInterval);
                            
                            // Set up refresh interval
                            refreshInterval = setInterval(async () => {{
                                await refreshContent();
                                // Restart the animations
                                restartAnimations();
                                
                                // Reset cycle start time
                                cycleStartTime = Date.now();
                            }}, REFRESH_INTERVAL_SECONDS * 1000);
                        }}
                        
                        // Manual refresh on button click
                        refreshButton.addEventListener('click', async () => {{
                            // Clear existing timers
                            if (refreshInterval) clearInterval(refreshInterval);
                            if (countdownInterval) clearInterval(countdownInterval);
                            
                            // Restart animations
                            restartAnimations();
                            
                            // Refresh immediately
                            await refreshContent();
                            
                            // Start the cycle again
                            startRefreshCycle();
                        }});
                        
                        // Start the initial refresh cycle
                        startRefreshCycle();
                    });
                "))
            }
        }
    }
}
