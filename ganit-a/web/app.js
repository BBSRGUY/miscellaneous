// Ganit-A Web Application
// API Configuration
const API_BASE = 'http://localhost:8000';
const WS_URL = 'ws://localhost:8000/ws';

// State
let currentView = 'dashboard';
let ws = null;
let discoveries = [];
let selectedDiscovery = null;
let charts = {};

// Initialize application
document.addEventListener('DOMContentLoaded', () => {
    initWebSocket();
    initCharts();
    loadDashboard();
    setInterval(refreshData, 30000); // Refresh every 30 seconds
});

// WebSocket Connection
function initWebSocket() {
    try {
        ws = new WebSocket(WS_URL);

        ws.onopen = () => {
            console.log('WebSocket connected');
            updateStatus(true);
        };

        ws.onmessage = (event) => {
            const message = JSON.parse(event.data);
            handleWebSocketMessage(message);
        };

        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
            updateStatus(false);
        };

        ws.onclose = () => {
            console.log('WebSocket disconnected');
            updateStatus(false);
            // Reconnect after 5 seconds
            setTimeout(initWebSocket, 5000);
        };
    } catch (error) {
        console.error('WebSocket connection failed:', error);
        updateStatus(false);
    }
}

function handleWebSocketMessage(message) {
    console.log('WebSocket message:', message);

    switch (message.type) {
        case 'run_started':
            showToast('Discovery run started!');
            showLoading(true);
            break;

        case 'run_completed':
            showToast(`Run completed! Found ${message.stats.total_discoveries} discoveries`);
            showLoading(false);
            refreshData();
            break;

        case 'run_error':
            showToast(`Error: ${message.error}`, 'error');
            showLoading(false);
            break;
    }
}

function updateStatus(connected) {
    const dot = document.getElementById('status-dot');
    const text = document.getElementById('status-text');

    if (connected) {
        dot.style.background = '#00ff88';
        text.textContent = 'Connected';
    } else {
        dot.style.background = '#ff3366';
        text.textContent = 'Disconnected';
    }
}

// View Management
function showView(viewName) {
    // Hide all views
    document.querySelectorAll('.view').forEach(view => {
        view.classList.remove('active');
    });

    // Show selected view
    document.getElementById(`${viewName}-view`).classList.add('active');

    // Update nav buttons
    document.querySelectorAll('.nav-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    event.target.classList.add('active');

    currentView = viewName;

    // Load view-specific data
    switch (viewName) {
        case 'dashboard':
            loadDashboard();
            break;
        case 'discoveries':
            loadAllDiscoveries();
            break;
        case 'explorer':
            loadExplorer();
            break;
    }
}

// Dashboard Functions
async function loadDashboard() {
    try {
        // Load latest run stats
        const runs = await fetchAPI('/runs');
        if (runs.length > 0) {
            const latestRun = runs[0];
            const stats = await fetchAPI(`/runs/${latestRun.run_id}/stats`);

            // Update stats
            document.getElementById('stat-total').textContent = stats.total_discoveries || 0;
            document.getElementById('stat-certified').textContent = stats.certified || 0;
            document.getElementById('stat-recognized').textContent = stats.recognized || 0;
            document.getElementById('stat-validated').textContent = stats.cross_validated || 0;

            // Load discoveries
            discoveries = await fetchAPI(`/discoveries?run_id=${latestRun.run_id}&limit=50`);

            // Update charts
            updateStageChart(discoveries);
            updateTimelineChart(discoveries);

            // Show top discoveries
            displayTopDiscoveries(discoveries.slice(0, 10));
        }
    } catch (error) {
        console.error('Error loading dashboard:', error);
    }
}

async function loadAllDiscoveries() {
    try {
        const stage = document.getElementById('stage-filter').value;
        const query = stage ? `?stage=${stage}` : '';

        discoveries = await fetchAPI(`/discoveries${query}&limit=100`);
        displayDiscoveriesGrid(discoveries);
    } catch (error) {
        console.error('Error loading discoveries:', error);
    }
}

function displayTopDiscoveries(topDiscoveries) {
    const container = document.getElementById('top-discoveries');
    container.innerHTML = '';

    topDiscoveries.forEach(disc => {
        const item = createDiscoveryItem(disc);
        container.appendChild(item);
    });
}

function displayDiscoveriesGrid(discoveriesList) {
    const container = document.getElementById('all-discoveries');
    container.innerHTML = '';

    discoveriesList.forEach(disc => {
        const card = createDiscoveryCard(disc);
        container.appendChild(card);
    });
}

function createDiscoveryItem(disc) {
    const item = document.createElement('div');
    item.className = 'discovery-item';
    item.onclick = () => showDiscoveryDetails(disc.discovery_id);

    const header = document.createElement('div');
    header.className = 'discovery-header';

    const id = document.createElement('span');
    id.className = 'discovery-id';
    id.textContent = `#${disc.discovery_id}`;

    const badge = document.createElement('span');
    badge.className = `discovery-badge badge-${disc.stage}`;
    badge.textContent = disc.stage;

    header.appendChild(id);
    header.appendChild(badge);

    const value = document.createElement('div');
    value.className = 'discovery-value';
    value.textContent = disc.value_repr.substring(0, 60) + (disc.value_repr.length > 60 ? '...' : '');

    const meta = document.createElement('div');
    meta.className = 'discovery-meta';
    meta.innerHTML = `
        <span>Method: ${disc.module_name}</span>
        <span>Score: ${disc.rank_score.toFixed(4)}</span>
        <span>Precision: ${disc.ulp_bits} bits</span>
    `;

    item.appendChild(header);
    item.appendChild(value);
    item.appendChild(meta);

    return item;
}

function createDiscoveryCard(disc) {
    const card = document.createElement('div');
    card.className = 'discovery-card';
    card.onclick = () => showDiscoveryDetails(disc.discovery_id);

    card.innerHTML = `
        <div class="discovery-header">
            <span class="discovery-id">#${disc.discovery_id}</span>
            <span class="discovery-badge badge-${disc.stage}">${disc.stage}</span>
        </div>
        <div class="discovery-value">${disc.value_repr.substring(0, 50)}${disc.value_repr.length > 50 ? '...' : ''}</div>
        <div class="discovery-meta">
            <span>Method: ${disc.module_name}</span>
            <span>Score: ${disc.rank_score.toFixed(4)}</span>
        </div>
    `;

    return card;
}

// Chart Initialization
function initCharts() {
    // Stage distribution chart
    const stageCtx = document.getElementById('stageChart').getContext('2d');
    charts.stage = new Chart(stageCtx, {
        type: 'doughnut',
        data: {
            labels: ['Observed', 'Certified', 'Recognized', 'Cross-Validated'],
            datasets: [{
                data: [0, 0, 0, 0],
                backgroundColor: [
                    'rgba(160, 168, 192, 0.5)',
                    'rgba(0, 255, 136, 0.5)',
                    'rgba(255, 170, 0, 0.5)',
                    'rgba(255, 0, 255, 0.5)'
                ],
                borderColor: [
                    '#a0a8c0',
                    '#00ff88',
                    '#ffaa00',
                    '#ff00ff'
                ],
                borderWidth: 2
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    labels: {
                        color: '#a0a8c0'
                    }
                }
            }
        }
    });

    // Timeline chart
    const timelineCtx = document.getElementById('timelineChart').getContext('2d');
    charts.timeline = new Chart(timelineCtx, {
        type: 'line',
        data: {
            labels: [],
            datasets: [{
                label: 'Discoveries',
                data: [],
                borderColor: '#00d4ff',
                backgroundColor: 'rgba(0, 212, 255, 0.1)',
                tension: 0.4,
                fill: true
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                y: {
                    beginAtZero: true,
                    ticks: {
                        color: '#a0a8c0'
                    },
                    grid: {
                        color: '#2a3050'
                    }
                },
                x: {
                    ticks: {
                        color: '#a0a8c0'
                    },
                    grid: {
                        color: '#2a3050'
                    }
                }
            },
            plugins: {
                legend: {
                    labels: {
                        color: '#a0a8c0'
                    }
                }
            }
        }
    });

    // Convergence chart
    const convCtx = document.getElementById('convergenceChart').getContext('2d');
    charts.convergence = new Chart(convCtx, {
        type: 'line',
        data: {
            labels: [],
            datasets: [{
                label: 'log₁₀(radius)',
                data: [],
                borderColor: '#00d4ff',
                backgroundColor: 'rgba(0, 212, 255, 0.1)',
                tension: 0.1,
                pointRadius: 2
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                y: {
                    ticks: {
                        color: '#a0a8c0'
                    },
                    grid: {
                        color: '#2a3050'
                    }
                },
                x: {
                    type: 'logarithmic',
                    ticks: {
                        color: '#a0a8c0'
                    },
                    grid: {
                        color: '#2a3050'
                    }
                }
            },
            plugins: {
                legend: {
                    labels: {
                        color: '#a0a8c0'
                    }
                }
            }
        }
    });
}

function updateStageChart(discoveriesList) {
    const stageCounts = {
        observed: 0,
        certified: 0,
        recognized: 0,
        cross_validated: 0
    };

    discoveriesList.forEach(disc => {
        if (stageCounts.hasOwnProperty(disc.stage)) {
            stageCounts[disc.stage]++;
        }
    });

    charts.stage.data.datasets[0].data = [
        stageCounts.observed,
        stageCounts.certified,
        stageCounts.recognized,
        stageCounts.cross_validated
    ];
    charts.stage.update();
}

function updateTimelineChart(discoveriesList) {
    // Group by creation date
    const dateGroups = {};

    discoveriesList.forEach(disc => {
        const date = disc.created_at ? disc.created_at.split('T')[0] : 'Unknown';
        dateGroups[date] = (dateGroups[date] || 0) + 1;
    });

    const labels = Object.keys(dateGroups).sort();
    const data = labels.map(label => dateGroups[label]);

    charts.timeline.data.labels = labels;
    charts.timeline.data.datasets[0].data = data;
    charts.timeline.update();
}

// Explorer Functions
async function loadExplorer() {
    if (discoveries.length > 0) {
        await showDiscoveryDetails(discoveries[0].discovery_id);
    }
}

async function showDiscoveryDetails(discoveryId) {
    try {
        const disc = await fetchAPI(`/discoveries/${discoveryId}`);
        const trace = await fetchAPI(`/discoveries/${discoveryId}/trace`);

        selectedDiscovery = disc;

        // Update details panel
        const detailsHtml = `
            <div class="discovery-detail">
                <h4>Discovery #${disc.discovery_id}</h4>
                <div class="detail-row">
                    <span class="label">Value:</span>
                    <span class="value monospace">${disc.value_repr.substring(0, 40)}...</span>
                </div>
                <div class="detail-row">
                    <span class="label">Method:</span>
                    <span class="value">${disc.module_name}</span>
                </div>
                <div class="detail-row">
                    <span class="label">Stage:</span>
                    <span class="discovery-badge badge-${disc.stage}">${disc.stage}</span>
                </div>
                <div class="detail-row">
                    <span class="label">Rank Score:</span>
                    <span class="value">${disc.rank_score.toFixed(6)}</span>
                </div>
                <div class="detail-row">
                    <span class="label">Precision:</span>
                    <span class="value">${disc.ulp_bits} bits</span>
                </div>
            </div>
        `;

        document.getElementById('explorer-details').innerHTML = detailsHtml;

        // Update convergence chart
        updateConvergenceChart(trace);

        // Show recognition if available
        if (disc.recognition_attempts && disc.recognition_attempts.length > 0) {
            const recog = disc.recognition_attempts[0];
            if (recog.success && recog.relation_tex) {
                const relationHtml = `
                    <h3>Recognized Relation</h3>
                    <div class="relation-tex">$$${recog.relation_tex}$$</div>
                    <div class="relation-meta">
                        <span>Height: ${recog.height}</span>
                        <span>Residual: 10^${recog.residual_log10.toFixed(2)}</span>
                    </div>
                `;
                document.getElementById('relation-display').innerHTML = relationHtml;

                // Re-render MathJax
                if (window.MathJax) {
                    MathJax.typeset();
                }
            }
        }

        // Switch to explorer view if not already there
        if (currentView !== 'explorer') {
            showView('explorer');
        }

    } catch (error) {
        console.error('Error loading discovery details:', error);
    }
}

function updateConvergenceChart(trace) {
    const labels = trace.map(sample => sample.n);
    const data = trace.map(sample => sample.log10_radius);

    charts.convergence.data.labels = labels;
    charts.convergence.data.datasets[0].data = data;
    charts.convergence.update();
}

// Run Management
async function startRun() {
    const config = {
        target_precision_digits: parseInt(document.getElementById('precision-input').value),
        wallclock_limit_minutes: parseFloat(document.getElementById('time-limit-input').value),
        max_workers: parseInt(document.getElementById('workers-input').value),
        random_seed: parseInt(document.getElementById('seed-input').value),
        modules: []
    };

    // Add enabled modules
    if (document.getElementById('module-series').checked) {
        config.modules.push({
            name: 'series_products',
            enabled: true,
            budget_minutes: config.wallclock_limit_minutes / 3
        });
    }

    if (document.getElementById('module-cf').checked) {
        config.modules.push({
            name: 'continued_fractions',
            enabled: true,
            budget_minutes: config.wallclock_limit_minutes / 3
        });
    }

    if (document.getElementById('module-fp').checked) {
        config.modules.push({
            name: 'fixed_points',
            enabled: true,
            budget_minutes: config.wallclock_limit_minutes / 3
        });
    }

    try {
        showLoading(true);
        const response = await fetchAPI('/runs/start', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(config)
        });

        showToast('Discovery run started!');
    } catch (error) {
        console.error('Error starting run:', error);
        showToast('Error starting run: ' + error.message, 'error');
        showLoading(false);
    }
}

// Utility Functions
async function fetchAPI(endpoint, options = {}) {
    const response = await fetch(API_BASE + endpoint, options);

    if (!response.ok) {
        throw new Error(`API error: ${response.statusText}`);
    }

    return response.json();
}

function showLoading(show) {
    const overlay = document.getElementById('loading-overlay');
    if (show) {
        overlay.classList.add('active');
    } else {
        overlay.classList.remove('active');
    }
}

function showToast(message, type = 'info') {
    const toast = document.getElementById('toast');
    toast.textContent = message;
    toast.classList.add('show');

    setTimeout(() => {
        toast.classList.remove('show');
    }, 3000);
}

function filterDiscoveries() {
    loadAllDiscoveries();
}

function refreshData() {
    if (currentView === 'dashboard') {
        loadDashboard();
    } else if (currentView === 'discoveries') {
        loadAllDiscoveries();
    }
}
