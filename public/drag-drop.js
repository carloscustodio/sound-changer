// InteractJS-based Drag & Drop for device tiles into fixed priority slots

function getPriorityLabel(slot) {
    const labels = ['Highest Priority', 'High Priority', 'Medium Priority', 'Low Priority', 'Lowest Priority'];
    return labels[slot] || 'Priority';
}

function renderFilledSlot(zone, slotNumber, device, isAvailable = true) {
    const label = getPriorityLabel(slotNumber);
    zone.className = 'priority-box filled' + (isAvailable ? ' available' : '');
    zone.innerHTML = `
        <div class="priority-header">
            <div class="priority-number">${slotNumber + 1}</div>
            <div class="priority-label">${label}</div>
        </div>
        <div class="device-info">
            <div class="device-name">${device.device_name}</div>
            <div class="device-status">${isAvailable ? '✓ Available' : '✗ Disconnected'}</div>
        </div>
        <div class="priority-actions">
            <button class="priority-btn remove" title="Remove">✕</button>
        </div>
    `;

    const removeBtn = zone.querySelector('.priority-btn.remove');
    removeBtn?.addEventListener('click', () => {
        try {
            const chainType = zone.getAttribute('data-chain-type');
            const key = chainType === 'playback' ? 'playback_priorities' : 'recording_priorities';
            const raw = localStorage.getItem(key);
            let arr;
            try {
                arr = JSON.parse(raw ?? '[]');
            } catch (_) {
                arr = [];
            }
            if (!Array.isArray(arr)) arr = [];
            arr = arr.filter(e => e && e.priority !== slotNumber);
            localStorage.setItem(key, JSON.stringify(arr));
        } catch (e) {
            console.warn('Failed to update localStorage on remove:', e);
        }

        // Render empty slot again
        zone.className = 'priority-box empty';
        zone.innerHTML = `
            <div class="priority-header">
                <div class="priority-number">${slotNumber + 1}</div>
                <div class="priority-label">${label}</div>
            </div>
            <div class="empty-slot-content">
                <div class="drop-hint">Drop device here</div>
                <div class="drop-icon">📥</div>
            </div>
        `;
    });
}

// Make device tiles draggable
if (window.interact) {
    // Enable dynamic drop updates for elements added after init
    interact.dynamicDrop(true);
        interact('.draggable-tile')
            .draggable({
                inertia: true,
                autoScroll: true,
                listeners: {
                    start(event) {
                        const el = event.target;
                        el.classList.add('dragging');
                        el.style.opacity = '0.8';
                                el.style.zIndex = '1000';
                                console.log('Drag start', {
                                    id: el.getAttribute('data-device-id'),
                                    name: el.getAttribute('data-device-name'),
                                    type: el.getAttribute('data-device-type')
                                });
                        // cache position
                        el._dx = 0;
                        el._dy = 0;
                    },
                    move(event) {
                        const el = event.target;
                        el._dx = (el._dx || 0) + event.dx;
                        el._dy = (el._dy || 0) + event.dy;
                      el.style.zIndex = '';
                        el.style.transform = `translate(${el._dx}px, ${el._dy}px)`;
                    },
                    end(event) {
                        const el = event.target;
                        el.classList.remove('dragging');
                        el.style.opacity = '';
                        // reset transform unless dropped
                      console.log('Drag end');
                        if (!el._dropped) {
                            el.style.transform = '';
                            el._dx = 0;
                            el._dy = 0;
                        }
                        el._dropped = false;
                    }
                }
        });

    // Make each priority box a dropzone
        interact('[data-priority-slot]')
                .dropzone({
                    accept: '.draggable-tile',
                    overlap: 0.05,
            ondragenter(event) {
                    const zone = (event.target.matches('[data-priority-slot]') ? event.target : event.target.closest('[data-priority-slot]'));
                if (zone) {
                    zone.classList.add('drop-target');
                            zone.style.backgroundColor = 'rgba(0, 123, 255, 0.1)';
                            zone.style.border = '2px dashed #007bff';
                }
            },
            ondragleave(event) {
                    const zone = (event.target.matches('[data-priority-slot]') ? event.target : event.target.closest('[data-priority-slot]'));
                if (zone) {
                    zone.classList.remove('drop-target');
                    zone.style.backgroundColor = '';
                    zone.style.border = '';
                }
            },
            ondrop(event) {
                const tile = event.relatedTarget;
                    const zone = (event.target.matches('[data-priority-slot]') ? event.target : event.target.closest('[data-priority-slot]'));
                if (!tile || !zone) return;

                tile._dropped = true;
                tile.style.transform = '';
                tile._dx = 0;
                tile._dy = 0;

                const chainType = zone.getAttribute('data-chain-type');
                const slotNumber = parseInt(zone.getAttribute('data-priority-slot'));
                const device = {
                    device_id: tile.getAttribute('data-device-id'),
                    device_name: tile.getAttribute('data-device-name'),
                    device_type: tile.getAttribute('data-device-type'),
                };

            console.log('Drop onto', { chainType, slotNumber, device });
            // Validate type
                if (!device.device_type || !chainType ||
                        !((chainType === 'playback' && device.device_type === 'Playback') ||
                            (chainType === 'recording' && device.device_type === 'Recording'))) {
                    console.warn('Type mismatch: cannot drop', device.device_type, 'into', chainType);
                    return;
                }

                    // Update localStorage immediately (robust parse)
                        try {
                            const key = chainType === 'playback' ? 'playback_priorities' : 'recording_priorities';
                            const raw = localStorage.getItem(key);
                            let arr;
                            try {
                                arr = JSON.parse(raw ?? '[]');
                            } catch (_) {
                                arr = [];
                            }
                            if (!Array.isArray(arr)) arr = [];
                            // remove existing entry for this device or slot
                            arr = arr.filter(e => e && e.device_id !== device.device_id && e.priority !== slotNumber);
                            arr.push({
                                device_id: device.device_id,
                                device_name: device.device_name,
                                device_type: device.device_type,
                                priority: slotNumber
                            });
                            localStorage.setItem(key, JSON.stringify(arr));
                        } catch (e) {
                            console.warn('Failed to update localStorage priorities:', e);
                        }

                    // Update the UI immediately (no page reload)
                    renderFilledSlot(zone, slotNumber, device, true);

                            // If user dropped into a chain, also switch the OS default to this device now
                                            if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
                                                window.__TAURI__.core.invoke('set_default_device', {
                                                    // wrap under the parameter name expected by the Rust command (args)
                                                    args: {
                                                        device_id: device.device_id,
                                                        device_type: device.device_type,
                                                    }
                                                }).then(() => {
                                            console.log('Switched default device to', device);
                                            showToast(`Switched default device to ${device.device_name}`, 'success', 2000);
                                            // Optional: refresh once to reflect default badges/state
                                            setTimeout(() => { location.reload(); }, 600);
                                }).catch(err => {
                                            console.warn('Failed to set default device:', err);
                                            showToast('Failed to set default device', 'error', 2500);
                                });
                            }

                            // Invoke backend to log/persist server-side if implemented
                if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
                    window.__TAURI__.core.invoke('add_device_to_priority_slot', {
                        // wrap under the parameter name expected by the Rust command (args)
                        args: {
                            device_id: device.device_id,
                            device_name: device.device_name,
                            device_type: device.device_type,
                            priority_type: chainType,
                            priority_slot: slotNumber,
                        }
                    }).then(() => {
                        console.log('Added to slot', slotNumber, device);
                    }).catch(err => {
                        console.error('Failed to persist slot assignment:', err);
                    });
                } else {
                    console.log('Assign (dev only):', chainType, slotNumber, device);
                }
            }
        });
}

console.log('InteractJS drag-drop initialized');

// Improve touch behavior on tiles
document.addEventListener('DOMContentLoaded', () => {
    document.querySelectorAll('.draggable-tile').forEach(el => {
        el.style.touchAction = 'none';
    });
});

// Lightweight toast notifications
function ensureToastContainer() {
    let c = document.querySelector('.toast-container');
    if (!c) {
        c = document.createElement('div');
        c.className = 'toast-container';
        c.setAttribute('aria-live', 'polite');
        c.setAttribute('aria-atomic', 'true');
        document.body.appendChild(c);
    }
    return c;
}

function showToast(message, type = 'info', duration = 3000) {
    const container = ensureToastContainer();
    const t = document.createElement('div');
    t.className = `toast ${type}`;
    t.textContent = message;
    container.appendChild(t);
    requestAnimationFrame(() => t.classList.add('visible'));
    setTimeout(() => {
        t.classList.remove('visible');
        setTimeout(() => t.remove(), 300);
    }, duration);
}
