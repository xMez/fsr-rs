let ws = null;
const messagesDiv = document.getElementById('messages');
let currentProfiles = { profiles: {}, current_profile: '' };
let messageQueue = [];
let isProcessingQueue = false;
let reconnectAttempts = 0;
let maxReconnectAttempts = 10;
let reconnectDelay = 1000; // Start with 1 second
let reconnectTimeout = null;
let isReconnecting = false;
// Track current subscriptions (default: all event types for backward compatibility)
let currentSubscriptions = new Set(['command_response', 'sensor_stream', 'active_player_broadcast']);

function connectWebSocket() {
    if (isReconnecting) {
        addMessage('System', 'Connection attempt blocked - already reconnecting', 'error');
        return;
    }

    try {
        addMessage('System', 'Creating new WebSocket connection...', 'error');
        // Use relative WebSocket URL to connect to the same server that serves this page
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;
        ws = new WebSocket(wsUrl);
        setupWebSocketHandlers();
    } catch (error) {
        console.error('Failed to create WebSocket connection:', error);
        addMessage('System', 'Failed to create WebSocket connection', 'error');
        isReconnecting = false; // Reset flag so we can try again
        scheduleReconnect();
    }
}

function setupWebSocketHandlers() {
    ws.onopen = function () {
        console.log('Connected to WebSocket server');
        addMessage('System', 'Connected to server', 'success');

        // Reset reconnection state
        reconnectAttempts = 0;
        reconnectDelay = 1000;
        isReconnecting = false;

        // Update status to show connected but not yet receiving stream
        document.getElementById('streamStatusBar').className = 'stream-status-bar inactive';
        document.getElementById('streamStatus').textContent = 'Connected - waiting for stream';
        document.getElementById('reconnectBtn').style.display = 'none';

        setupThresholdBars();

        // Explicitly subscribe to all event types (server defaults to all, but this ensures sync)
        const subscribeCommand = {
            Subscribe: {
                event_types: ['command_response', 'sensor_stream', 'active_player_broadcast']
            }
        };
        ws.send(JSON.stringify(subscribeCommand));
        addMessage('System', 'Synced subscriptions: command_response, sensor_stream, active_player_broadcast', 'success');

        // Update subscription button states (if in debug mode)
        updateSubscriptionButtons();

        // Process any queued messages
        processQueuedMessages();

        // Start the sensor stream automatically
        startSensorStream();
    };

    ws.onmessage = function (event) {
        const response = JSON.parse(event.data);
        addMessage('Server', response.message, response.success ? 'success' : 'error');

        // Only update profiles UI if profiles data changed
        if (response.data) {
            const profilesChanged =
                currentProfiles.current_profile !== response.data.current_profile ||
                JSON.stringify(currentProfiles.profiles) !== JSON.stringify(response.data.profiles);
            currentProfiles = response.data;
            if (profilesChanged) {
                updateProfilesOnly();
                updateThresholdsOnly();
            }
        }

        // Update sensor values and labels atomically
        if (response.sensor_values) {
            // Use requestAnimationFrame to ensure smooth, synchronized updates
            requestAnimationFrame(() => {
                updateSensorValueBars(response.sensor_values);
            });
        }

        // Handle sensor stream data specifically
        if (response.response_type === 'sensor_stream' && response.sensor_values) {
            // This is automatic 60Hz sensor data - no need to log every update
            // Just update the UI smoothly
            requestAnimationFrame(() => {
                updateSensorValueBars(response.sensor_values);
            });

            // Ensure status indicator shows active when receiving data
            document.getElementById('streamStatusBar').className = 'stream-status-bar active';
            document.getElementById('streamStatus').textContent = 'Receiving 60Hz sensor stream';
        }

        // Handle stream stop confirmation from server
        if (response.response_type === 'stream_stopped' ||
            (response.message && response.message.toLowerCase().includes('sensor stream stopped')) ||
            (response.message && response.message.toLowerCase().includes('stream stopped'))) {
            document.getElementById('streamStatusBar').className = 'stream-status-bar inactive';
            document.getElementById('streamStatus').textContent = 'Stream stopped';
        }

        // Handle active player broadcast
        if (response.response_type === 'active_player_broadcast') {
            // Update the active player display without logging every broadcast
            updateActivePlayerDisplay(response.data);
        }

        // Handle subscription confirmations
        if (response.message && response.message.includes('Subscribed to:')) {
            // Extract event types from message and update UI
            const eventTypes = response.message.split('Subscribed to:')[1].trim().split(', ');
            eventTypes.forEach(type => currentSubscriptions.add(type.trim()));
            updateSubscriptionButtons();
        } else if (response.message && response.message.includes('Unsubscribed from:')) {
            // Extract event types from message and update UI
            const eventTypes = response.message.split('Unsubscribed from:')[1].trim().split(', ');
            eventTypes.forEach(type => currentSubscriptions.delete(type.trim()));
            updateSubscriptionButtons();
        }
    };

    ws.onclose = function (event) {
        addMessage('System', 'Disconnected from server', 'error');

        // Update status to show disconnected
        document.getElementById('streamStatusBar').className = 'stream-status-bar inactive';
        document.getElementById('streamStatus').textContent = 'Disconnected';

        // Show reconnect button
        document.getElementById('reconnectBtn').style.display = 'inline-block';

        // Attempt reconnection for any disconnection (clean or not)
        if (reconnectAttempts < maxReconnectAttempts) {
            scheduleReconnect();
        }
    };

    ws.onerror = function (error) {
        console.error('WebSocket error:', error);
        addMessage('System', 'WebSocket error occurred', 'error');

        // Update status to show error state
        document.getElementById('streamStatusBar').className = 'stream-status-bar inactive';
        document.getElementById('streamStatus').textContent = 'Connection Error';
    };
}

function scheduleReconnect() {
    if (isReconnecting || reconnectAttempts >= maxReconnectAttempts) {
        addMessage('System', `Reconnection blocked: isReconnecting=${isReconnecting}, attempts=${reconnectAttempts}/${maxReconnectAttempts}`, 'error');

        // If we've exhausted all reconnection attempts, update status
        if (reconnectAttempts >= maxReconnectAttempts) {
            document.getElementById('streamStatusBar').className = 'stream-status-bar inactive';
            document.getElementById('streamStatus').textContent = 'Connection Failed';
        }
        return;
    }

    isReconnecting = true;
    reconnectAttempts++;

    addMessage('System', `Attempting to reconnect... (${reconnectAttempts}/${maxReconnectAttempts})`, 'error');

    // Clear any existing timeout
    if (reconnectTimeout) {
        clearTimeout(reconnectTimeout);
    }

    // Exponential backoff with max delay of 30 seconds
    const delay = Math.min(reconnectDelay * Math.pow(2, reconnectAttempts - 1), 30000);

    addMessage('System', `Will attempt reconnection in ${delay}ms`, 'error');

    reconnectTimeout = setTimeout(() => {
        addMessage('System', 'Reconnecting to server...', 'error');
        isReconnecting = false; // Reset flag before attempting connection
        connectWebSocket();
    }, delay);
}

function processQueuedMessages() {
    if (messageQueue.length > 0 && ws && ws.readyState === WebSocket.OPEN) {
        addMessage('System', `Processing ${messageQueue.length} queued messages...`, 'success');

        while (messageQueue.length > 0) {
            const command = messageQueue.shift();
            setTimeout(() => {
                ws.send(JSON.stringify(command));
            }, 10);
        }
    }
}

function manualReconnect() {
    addMessage('System', 'Manual reconnection initiated', 'success');

    // Reset reconnection state
    reconnectAttempts = 0;
    isReconnecting = false;

    // Clear any existing timeout
    if (reconnectTimeout) {
        clearTimeout(reconnectTimeout);
    }

    // Attempt immediate reconnection
    connectWebSocket();
}

// Initialize the WebSocket connection
connectWebSocket();

// Prevent double-tap zoom on buttons (iOS Safari and some Android browsers)
document.addEventListener('dblclick', function (e) {
    const target = e.target;
    if (target && (target.tagName === 'BUTTON' || target.closest('button'))) {
        e.preventDefault();
    }
}, { passive: false });

function addMessage(sender, message, type) {
    // Only show messages in debug mode
    if (window.DEBUG_MODE && messagesDiv) {
        const messageDiv = document.createElement('div');
        messageDiv.className = `message ${type}`;
        messageDiv.innerHTML = `<strong>${sender}:</strong> ${message}`;
        messagesDiv.appendChild(messageDiv);
        messagesDiv.scrollTop = messagesDiv.scrollHeight;
    }
}

function updateUI() {
    updateProfilesList();
    updateThresholdBarsFromProfiles();
}

function updateProfilesOnly() {
    updateProfilesList();
}

function updateThresholdsOnly() {
    updateThresholdBarsFromProfiles();
}

function updateProfilesList() {
    const profilesList = document.getElementById('profilesList');
    profilesList.innerHTML = '';

    Object.entries(currentProfiles.profiles).forEach(([name, profile]) => {
        const div = document.createElement('div');
        div.className = `profile-item ${name === currentProfiles.current_profile ? 'current-profile' : ''}`;
        div.innerHTML = `
            <div class="profile-name">${name}${name === currentProfiles.current_profile ? ' (CURRENT)' : ''}</div>
            <div class="profile-thresholds">[${profile.thresholds.join(', ')}]</div>
            <div class="profile-actions">
                <button class="delete-btn" onclick="deleteProfile('${name}')">🗑️</button>
            </div>
        `;

        // Add click event to change profile (but not on delete button)
        const maybeChangeProfile = (e) => {
            const isDelete = e.target && (e.target.classList && e.target.classList.contains('delete-btn')) || (e.target.closest && e.target.closest('.delete-btn'));
            if (isDelete) return;
            if (name !== currentProfiles.current_profile) {
                const command = { ChangeProfile: { name } };
                sendCommand(command);
            }
        };

        div.addEventListener('click', (e) => {
            maybeChangeProfile(e);
        });

        // On touch devices, prevent the first tap from only triggering hover; act immediately
        div.addEventListener('touchend', (e) => {
            const isDelete = e.target && (e.target.classList && e.target.classList.contains('delete-btn')) || (e.target.closest && e.target.closest('.delete-btn'));
            if (isDelete) {
                // Allow default so the delete button's click fires
                return;
            }
            e.preventDefault();
            maybeChangeProfile(e);
        }, { passive: false });

        profilesList.appendChild(div);
    });
}



function addProfileFromInput() {
    const input = document.getElementById('newProfileNameInput');
    const name = input.value.trim();

    if (!name) {
        return;
    }

    // Get thresholds from current profile
    let thresholds = [100, 200, 300, 400]; // Default fallback
    if (currentProfiles.current_profile && currentProfiles.profiles[currentProfiles.current_profile]) {
        thresholds = currentProfiles.profiles[currentProfiles.current_profile].thresholds;
    }

    const command = {
        AddProfile: {
            name: name,
            thresholds: thresholds
        }
    };
    sendCommand(command);

    // Clear input
    input.value = '';
    input.focus();
}

function handleAddProfileKeypress(event) {
    if (event.key === 'Enter') {
        addProfileFromInput();
    }
}



function deleteProfile(profileName) {
    if (confirm(`Are you sure you want to delete profile "${profileName}"?`)) {
        const command = {
            RemoveProfile: {
                name: profileName
            }
        };
        sendCommand(command);
    }
}



function sendCommand(command) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        // Add a small delay to prevent message conflicts
        setTimeout(() => {
            ws.send(JSON.stringify(command));
        }, 10);
    } else {
        // If WebSocket is not ready, queue the command for later
        messageQueue.push(command);
        addMessage('System', 'Command queued - waiting for connection...', 'error');
    }
}

function getSensorValues() {
    const command = {
        GetSensorValues: null
    };
    sendCommand(command);
}

function startSensorStream() {
    const command = {
        StartSensorStream: null
    };
    sendCommand(command);
    document.getElementById('streamStatus').textContent = 'Receiving 60Hz sensor stream';
    document.getElementById('streamStatusBar').className = 'stream-status-bar active';
    addMessage('System', 'Started sensor stream', 'success');
}

function stopSensorStream() {
    const command = {
        StopSensorStream: null
    };
    sendCommand(command);
    document.getElementById('streamStatus').textContent = 'Stream stopped';
    document.getElementById('streamStatusBar').className = 'stream-status-bar inactive';
    addMessage('System', 'Stopped sensor stream', 'success');
}

// Threshold bar functionality
const MIN_VALUE = 0;
const MAX_VALUE = 1023;

function updateThresholdBar(index, value) {
    const line = document.getElementById(`thresholdLine${index}`);
    const valueDisplay = document.getElementById(`thresholdValue${index}`);

    // Convert value (0-1023) to position (0-100%)
    const percentage = ((MAX_VALUE - value) / (MAX_VALUE - MIN_VALUE)) * 100;
    line.style.top = `${percentage}%`;
    valueDisplay.textContent = value;
    valueDisplay.style.top = `${percentage}%`;
}

function valueFromPosition(bar, y) {
    const rect = bar.getBoundingClientRect();
    const relativeY = y - rect.top;
    const percentage = (relativeY / rect.height) * 100;
    const clampedPercentage = Math.max(0, Math.min(100, percentage));
    return Math.round(((100 - clampedPercentage) / 100) * (MAX_VALUE - MIN_VALUE) + MIN_VALUE);
}

function setupThresholdBars() {
    const mousemoveTimeouts = [null, null, null, null]; // Timeouts for each bar
    const mouseInside = [false, false, false, false]; // Track if mouse is inside each bar

    for (let i = 0; i < 4; i++) {
        const bar = document.getElementById(`thresholdBar${i}`);
        const line = document.getElementById(`thresholdLine${i}`);

        // Click event for immediate movement
        bar.addEventListener('click', (e) => {
            const value = valueFromPosition(bar, e.clientY);
            updateThresholdBar(i, value);
            updateThresholdOnServer(i, value);
        });

        // Mouse events for cursor line and value
        bar.addEventListener('mouseenter', () => {
            mouseInside[i] = true;
            const cursorLine = document.getElementById(`cursorLine${i}`);
            const cursorValue = document.getElementById(`cursorValue${i}`);
            cursorLine.style.display = 'block';
            cursorValue.style.display = 'block';
        });

        bar.addEventListener('mouseleave', () => {
            mouseInside[i] = false;

            // Clear any pending mousemove timeout
            if (mousemoveTimeouts[i]) {
                clearTimeout(mousemoveTimeouts[i]);
                mousemoveTimeouts[i] = null;
            }

            const cursorLine = document.getElementById(`cursorLine${i}`);
            const cursorValue = document.getElementById(`cursorValue${i}`);
            cursorLine.style.display = 'none';
            cursorValue.style.display = 'none';
        });

        // Throttle mousemove events for better performance
        bar.addEventListener('mousemove', (e) => {
            if (mousemoveTimeouts[i]) {
                return; // Skip if already processing
            }

            mousemoveTimeouts[i] = setTimeout(() => {
                // Check if mouse is still inside before updating
                if (!mouseInside[i]) {
                    mousemoveTimeouts[i] = null;
                    return;
                }

                const cursorLine = document.getElementById(`cursorLine${i}`);
                const cursorValue = document.getElementById(`cursorValue${i}`);
                const rect = bar.getBoundingClientRect();
                const relativeY = e.clientY - rect.top;
                const percentage = (relativeY / rect.height) * 100;
                const clampedPercentage = Math.max(0, Math.min(100, percentage));

                // Use requestAnimationFrame for smooth updates
                requestAnimationFrame(() => {
                    // Double-check if mouse is still inside before updating DOM
                    if (!mouseInside[i]) {
                        return;
                    }

                    // Update cursor line position
                    cursorLine.style.top = `${clampedPercentage}%`;
                    cursorLine.style.display = 'block';

                    // Calculate and display value at cursor position
                    const value = valueFromPosition(bar, e.clientY);
                    cursorValue.textContent = value;
                    cursorValue.style.top = `${clampedPercentage}%`;
                    cursorValue.style.display = 'block';
                });

                mousemoveTimeouts[i] = null;
            }, 16); // ~60fps throttling
        });
    }
}

function updateThresholdOnServer(index, value) {
    if (currentProfiles.current_profile && currentProfiles.profiles[currentProfiles.current_profile]) {
        const command = {
            UpdateThreshold: {
                profile_name: currentProfiles.current_profile,
                threshold_index: index,
                value: value
            }
        };
        sendCommand(command);
    }
}

function incrementThreshold(index) {
    if (currentProfiles.current_profile && currentProfiles.profiles[currentProfiles.current_profile]) {
        const currentValue = currentProfiles.profiles[currentProfiles.current_profile].thresholds[index];
        const newValue = Math.min(1023, currentValue + 1); // Increment by 1, max 1023
        updateThresholdBar(index, newValue);
        updateThresholdOnServer(index, newValue);
    }
}

function decrementThreshold(index) {
    if (currentProfiles.current_profile && currentProfiles.profiles[currentProfiles.current_profile]) {
        const currentValue = currentProfiles.profiles[currentProfiles.current_profile].thresholds[index];
        const newValue = Math.max(0, currentValue - 1); // Decrement by 1, min 0
        updateThresholdBar(index, newValue);
        updateThresholdOnServer(index, newValue);
    }
}

function updateThresholdBarsFromProfiles() {
    if (currentProfiles.current_profile && currentProfiles.profiles[currentProfiles.current_profile]) {
        const profile = currentProfiles.profiles[currentProfiles.current_profile];
        for (let i = 0; i < 4; i++) {
            updateThresholdBar(i, profile.thresholds[i]);
        }
    }
}

function updateSensorValueBars(sensorValues) {
    // Batch all DOM updates to prevent visual inconsistencies
    const updates = [];

    for (let i = 0; i < 4; i++) {
        const sensorBar = document.getElementById(`sensorValueBar${i}`);
        const sensorValueLabel = document.getElementById(`sensorValueLabel${i}`);
        const sensorValue = sensorValues[i];

        // Calculate height percentage
        const heightPercentage = (sensorValue / MAX_VALUE) * 100;

        // Prepare all updates
        updates.push(() => {
            sensorBar.style.height = `${heightPercentage}%`;
            sensorValueLabel.textContent = sensorValue;

            // Update threshold exceeded state with smooth transitions
            if (currentProfiles.current_profile && currentProfiles.profiles[currentProfiles.current_profile]) {
                const threshold = currentProfiles.profiles[currentProfiles.current_profile].thresholds[i];
                if (sensorValue > threshold) {
                    sensorBar.classList.add('exceeded');
                } else {
                    sensorBar.classList.remove('exceeded');
                }
            }
        });
    }

    // Apply all updates in a single batch for smooth 60Hz updates
    updates.forEach(update => update());
}

function updateActivePlayerDisplay(profilesData) {
    if (profilesData && profilesData.current_player) {
        // Update the active player display
        const activePlayerElement = document.getElementById('activePlayer');
        if (activePlayerElement) {
            activePlayerElement.textContent = profilesData.current_player || 'None';
        }
    }
}

function changePlayerFromInput() {
    const input = document.getElementById('newPlayerNameInput');
    const playerName = input.value.trim();

    if (playerName) {
        const command = {
            ChangePlayer: {
                name: playerName
            }
        };
        sendCommand(command);
        input.value = '';
    }
}

function handleChangePlayerKeypress(event) {
    if (event.key === 'Enter') {
        changePlayerFromInput();
    }
}

// Subscription management functions
function toggleSubscription(eventType) {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
        addMessage('System', 'Cannot toggle subscription: WebSocket not connected', 'error');
        return;
    }

    const isSubscribed = currentSubscriptions.has(eventType);

    if (isSubscribed) {
        // Unsubscribe
        const command = {
            Unsubscribe: {
                event_types: [eventType]
            }
        };
        ws.send(JSON.stringify(command));
        currentSubscriptions.delete(eventType);
        addMessage('System', `Unsubscribing from: ${eventType}`, 'success');
    } else {
        // Subscribe
        const command = {
            Subscribe: {
                event_types: [eventType]
            }
        };
        ws.send(JSON.stringify(command));
        currentSubscriptions.add(eventType);
        addMessage('System', `Subscribing to: ${eventType}`, 'success');
    }

    updateSubscriptionButtons();
}

function updateSubscriptionButtons() {
    // Only update if we're in debug mode (subscription controls exist)
    if (!window.DEBUG_MODE) {
        return;
    }

    // Map event types to button IDs and display names
    const buttonMap = {
        'command_response': { id: 'subCommandResponse', name: 'Command Response' },
        'sensor_stream': { id: 'subSensorStream', name: 'Sensor Stream' },
        'active_player_broadcast': { id: 'subActivePlayer', name: 'Active Player' }
    };

    // Update button states
    Object.entries(buttonMap).forEach(([eventType, config]) => {
        const button = document.getElementById(config.id);
        if (button) {
            if (currentSubscriptions.has(eventType)) {
                button.classList.add('active');
                button.textContent = config.name;
            } else {
                button.classList.remove('active');
                button.textContent = config.name + ' (off)';
            }
        }
    });
} 