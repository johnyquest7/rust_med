// UI Manager Module - Handles all user interface interactions and updates
import { WaveformVisualizer } from './waveformVisualizer.js';

export class UIManager {
    constructor() {
        this.elements = {};
        this.recordingTimer = null;
        this.recordingStartTime = null;
        this.pausedTime = null;
        this.availableDevices = [];
        this.selectedDeviceId = null;
        this.waveformVisualizer = null;
        this.initializeElements();
        this.createDynamicElements();
    }

    initializeElements() {
        this.elements = {
            startBtn: document.getElementById('startBtn'),
            stopBtn: document.getElementById('stopBtn'),
            pauseBtn: document.getElementById('pauseBtn'),
            saveBtn: document.getElementById('saveBtn'),
            status: document.getElementById('status'),
            transcript: document.getElementById('transcript'),
            medicalNote: document.getElementById('medicalNote'),
            copyBtn: document.getElementById('copyBtn'),
            recordingDot: document.getElementById('recordingDot'),
            recordingText: document.getElementById('recordingText'),
            noteType: document.getElementById('noteType'),
            // Patient info fields
            firstName: document.getElementById('firstName'),
            lastName: document.getElementById('lastName'),
            dob: document.getElementById('dob'),
            // Sidebar elements
            sidebar: document.getElementById('sidebar'),
            sidebarToggle: document.getElementById('sidebarToggle'),
            notesList: document.getElementById('notesList'),
            // Settings panel elements
            settingsPanel: document.getElementById('settingsPanel'),
            settingsToggle: document.getElementById('settingsToggle')
        };
        
        this.setupSidebarToggle();
        this.setupSettingsToggle();
    }

    createDynamicElements() {
        this.createTimerDisplay();
        this.createMicrophoneSelector();
        this.initializeWaveform();
    }
    
    initializeWaveform() {
        // Initialize waveform visualizer
        this.waveformVisualizer = new WaveformVisualizer('waveform');
    }

    createTimerDisplay() {
        const recordingControls = document.querySelector('.recording-controls');
        if (!recordingControls) return;

        const timerContainer = document.createElement('div');
        timerContainer.style.cssText = `
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 10px 0;
            font-size: 18px;
            font-weight: bold;
            color: #333;
        `;

        this.elements.timerDisplay = document.createElement('div');
        this.elements.timerDisplay.textContent = '00:00';
        this.elements.timerDisplay.style.cssText = `
            padding: 8px 16px;
            background: rgba(255, 255, 255, 0.9);
            border-radius: 8px;
            border: 2px solid #ddd;
            min-width: 80px;
            text-align: center;
        `;

        timerContainer.appendChild(this.elements.timerDisplay);
        recordingControls.appendChild(timerContainer);
    }

    createMicrophoneSelector() {
        const recordingControls = document.querySelector('.recording-controls');
        if (!recordingControls) return;

        const micContainer = document.createElement('div');
        micContainer.style.cssText = `
            display: flex;
            align-items: center;
            gap: 10px;
            margin-top: 15px;
            padding: 10px;
            background: rgba(255, 255, 255, 0.8);
            border-radius: 10px;
            flex-wrap: wrap;
        `;

        const micLabel = document.createElement('label');
        micLabel.textContent = 'Microphone:';
        micLabel.style.fontWeight = 'bold';

        this.elements.micSelect = document.createElement('select');
        this.elements.micSelect.style.cssText = `
            padding: 5px 10px;
            border: 1px solid #ccc;
            border-radius: 5px;
            background: white;
            min-width: 200px;
        `;

        this.elements.testBtn = document.createElement('button');
        this.elements.testBtn.textContent = 'Test Mic';
        this.elements.testBtn.className = 'btn';
        this.elements.testBtn.style.cssText = `
            padding: 5px 15px;
            background: #17a2b8;
            color: white;
            border: none;
            border-radius: 5px;
            cursor: pointer;
        `;

        micContainer.appendChild(micLabel);
        micContainer.appendChild(this.elements.micSelect);
        micContainer.appendChild(this.elements.testBtn);

        recordingControls.appendChild(micContainer);

        // Add event listeners
        this.elements.micSelect.addEventListener('change', (e) => {
            this.selectedDeviceId = e.target.value;
        });
    }

    async loadMicrophones() {
        try {
            await navigator.mediaDevices.getUserMedia({ audio: true });
            const devices = await navigator.mediaDevices.enumerateDevices();
            this.availableDevices = devices.filter(device => device.kind === 'audioinput');

            if (this.elements.micSelect) {
                this.elements.micSelect.innerHTML = '';
                
                if (this.availableDevices.length === 0) {
                    const option = document.createElement('option');
                    option.value = '';
                    option.textContent = 'No microphones found';
                    this.elements.micSelect.appendChild(option);
                    return;
                }

                this.availableDevices.forEach((device, index) => {
                    const option = document.createElement('option');
                    option.value = device.deviceId;
                    option.textContent = device.label || `Microphone ${index + 1}`;
                    this.elements.micSelect.appendChild(option);
                });

                if (this.availableDevices.length > 0) {
                    this.selectedDeviceId = this.availableDevices[0].deviceId;
                    this.elements.micSelect.value = this.selectedDeviceId;
                }
            }

            this.updateStatus(`Found ${this.availableDevices.length} microphone(s)`);

        } catch (error) {
            console.error('Error loading microphones:', error);
            this.showError('Could not access microphones: ' + error.message);
        }
    }

    async testMicrophone() {
        if (!this.selectedDeviceId) {
            this.showError('Please select a microphone first');
            return;
        }

        try {
            this.updateStatus('Testing microphone...');
            
            const stream = await navigator.mediaDevices.getUserMedia({
                audio: {
                    deviceId: this.selectedDeviceId ? { exact: this.selectedDeviceId } : undefined,
                    sampleRate: 44100,
                    channelCount: 1,
                    echoCancellation: true,
                    noiseSuppression: true,
                    autoGainControl: true
                }
            });

            const audioContext = new AudioContext();
            const analyser = audioContext.createAnalyser();
            const microphone = audioContext.createMediaStreamSource(stream);
            
            analyser.fftSize = 256;
            const dataArray = new Uint8Array(analyser.frequencyBinCount);
            
            microphone.connect(analyser);

            let maxLevel = 0;
            const monitorInterval = setInterval(() => {
                analyser.getByteFrequencyData(dataArray);
                const level = Math.max(...dataArray);
                maxLevel = Math.max(maxLevel, level);
                this.updateStatus(`Testing... Audio level: ${level}/255 (max: ${maxLevel})`);
            }, 100);

            setTimeout(() => {
                clearInterval(monitorInterval);
                stream.getTracks().forEach(track => track.stop());
                audioContext.close();

                if (maxLevel < 10) {
                    this.showError('Very low audio signal detected. Check microphone connection and volume.');
                } else if (maxLevel < 50) {
                    this.updateStatus('Low audio signal. Consider speaking louder or adjusting microphone position.');
                } else {
                    this.updateStatus(`Microphone test passed! Maximum level: ${maxLevel}/255`);
                }
            }, 3000);

        } catch (error) {
            console.error('Microphone test failed:', error);
            this.showError('Microphone test failed: ' + error.message);
        }
    }

    startTimer() {
        if (!this.recordingStartTime) {
            this.recordingStartTime = Date.now();
        } else if (this.pausedTime) {
            // Resume from paused time
            this.recordingStartTime += (Date.now() - this.pausedTime);
            this.pausedTime = null;
        }
        
        this.recordingTimer = setInterval(() => {
            const elapsed = Date.now() - this.recordingStartTime;
            const minutes = Math.floor(elapsed / 60000);
            const seconds = Math.floor((elapsed % 60000) / 1000);
            const display = `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
            
            if (this.elements.timerDisplay) {
                this.elements.timerDisplay.textContent = display;
                this.elements.timerDisplay.style.borderColor = '#ff4757';
                this.elements.timerDisplay.style.background = '#ffe6e6';
            }
        }, 1000);
    }

    stopTimer(isPause = false) {
        if (this.recordingTimer) {
            clearInterval(this.recordingTimer);
            this.recordingTimer = null;
        }
        
        if (isPause) {
            this.pausedTime = Date.now();
        } else {
            // Full stop - reset everything
            this.recordingStartTime = null;
            this.pausedTime = null;
            
            if (this.elements.timerDisplay) {
                this.elements.timerDisplay.style.borderColor = '#ddd';
                this.elements.timerDisplay.style.background = 'rgba(255, 255, 255, 0.9)';
            }
        }
    }

    updateRecordingState(isRecording, isPaused = false) {
        if (this.elements.startBtn) this.elements.startBtn.disabled = isRecording;
        if (this.elements.stopBtn) this.elements.stopBtn.disabled = !isRecording;
        if (this.elements.pauseBtn) {
            this.elements.pauseBtn.disabled = !isRecording;
            this.elements.pauseBtn.textContent = isPaused ? 'Resume' : 'Pause';
        }
        if (this.elements.micSelect) this.elements.micSelect.disabled = isRecording;
        
        if (this.elements.recordingDot) {
            if (isRecording && !isPaused) {
                this.elements.recordingDot.classList.add('active');
            } else {
                this.elements.recordingDot.classList.remove('active');
            }
        }
        
        if (this.elements.recordingText) {
            if (!isRecording) {
                this.elements.recordingText.textContent = 'Not Recording';
            } else if (isPaused) {
                this.elements.recordingText.textContent = 'Paused';
            } else {
                this.elements.recordingText.textContent = 'Recording...';
            }
        }
    }

    clearResults() {
        if (this.elements.transcript) this.elements.transcript.innerHTML = '';
        if (this.elements.medicalNote) this.elements.medicalNote.value = '';
    }

    updateStatus(message) {
        if (this.elements.status) {
            this.elements.status.textContent = message;
            this.elements.status.style.color = '';
        }
    }

    showError(message) {
        console.error('Error:', message);
        if (this.elements.status) {
            this.elements.status.textContent = message;
            this.elements.status.style.color = '#ff4757';
            setTimeout(() => {
                this.updateStatus('Ready');
            }, 5000);
        }
    }

    showTranscript(transcript) {
        if (this.elements.transcript) {
            this.elements.transcript.textContent = transcript;
            // Add a subtle animation when text updates
            this.elements.transcript.style.animation = 'fadeIn 0.3s ease-in';
            setTimeout(() => {
                this.elements.transcript.style.animation = '';
            }, 300);
        }
    }

    showMedicalNote(note) {
        if (this.elements.medicalNote) {
            this.elements.medicalNote.value = note;
            // Add visual feedback for note generation
            if (note.includes('Generating')) {
                this.elements.medicalNote.style.background = '#f0f8ff';
                this.elements.medicalNote.style.color = '#666';
                this.elements.medicalNote.style.fontStyle = 'italic';
            } else {
                this.elements.medicalNote.style.background = '#f8f9fa';
                this.elements.medicalNote.style.color = 'inherit';
                this.elements.medicalNote.style.fontStyle = 'normal';
            }
        }
    }

    showStandaloneResults(convertedBlob, timerText, selectedMicName) {
        const convertedUrl = URL.createObjectURL(convertedBlob);
        
        if (this.elements.transcript) {
            this.elements.transcript.innerHTML = '';
            
            const convertedLabel = document.createElement('div');
            convertedLabel.textContent = 'Converted Audio (WAV - compatible with whisperfile):';
            convertedLabel.style.fontWeight = 'bold';
            convertedLabel.style.color = 'green';
            
            const convertedPlayer = document.createElement('audio');
            convertedPlayer.controls = true;
            convertedPlayer.src = convertedUrl;
            convertedPlayer.style.display = 'block';
            convertedPlayer.style.marginTop = '5px';
            convertedPlayer.style.width = '100%';
            
            const downloadLink = document.createElement('a');
            downloadLink.href = convertedUrl;
            downloadLink.download = 'recording.wav';
            downloadLink.textContent = 'Download WAV file for manual transcription';
            downloadLink.style.display = 'block';
            downloadLink.style.marginTop = '10px';
            downloadLink.style.color = '#007bff';
            downloadLink.style.textDecoration = 'underline';
            
            const info = document.createElement('div');
            info.style.marginTop = '15px';
            info.style.padding = '10px';
            info.style.background = '#d4edda';
            info.style.borderRadius = '5px';
            info.innerHTML = `
                <strong>✅ Recording Complete!</strong><br>
                Duration: ${timerText}<br>
                Size: ${(convertedBlob.size / 1024).toFixed(1)} KB (WAV)<br>
                Sample Rate: 16kHz (optimized for speech recognition)<br>
                Device: ${selectedMicName}<br><br>
                <em>Download the WAV file above to use with whisperfile for transcription!</em>
            `;
            
            this.elements.transcript.appendChild(convertedLabel);
            this.elements.transcript.appendChild(convertedPlayer);
            this.elements.transcript.appendChild(downloadLink);
            this.elements.transcript.appendChild(info);
        }
    }

    async copyNote() {
        const noteContent = this.elements.medicalNote ? this.elements.medicalNote.value.trim() : '';
        if (!noteContent) {
            this.showError('No note to copy');
            return;
        }

        try {
            await navigator.clipboard.writeText(noteContent);
            this.updateStatus('Note copied to clipboard');
            setTimeout(() => {
                this.updateStatus('Ready');
            }, 2000);
        } catch (error) {
            console.error('Failed to copy to clipboard:', error);
            this.showError('Failed to copy note to clipboard');
        }
    }

    getSelectedDeviceId() {
        return this.selectedDeviceId;
    }

    getSelectedMicrophoneName() {
        const selectedDevice = this.availableDevices.find(device => device.deviceId === this.selectedDeviceId);
        return selectedDevice ? selectedDevice.label : 'Unknown microphone';
    }

    getTimerText() {
        return this.elements.timerDisplay ? this.elements.timerDisplay.textContent : 'Unknown';
    }

    // Event binding helpers
    bindStartButton(callback) {
        if (this.elements.startBtn) {
            this.elements.startBtn.addEventListener('click', callback);
        }
    }

    bindStopButton(callback) {
        if (this.elements.stopBtn) {
            this.elements.stopBtn.addEventListener('click', callback);
        }
    }

    bindCopyButton(callback) {
        if (this.elements.copyBtn) {
            this.elements.copyBtn.addEventListener('click', callback);
        }
    }

    bindTestButton(callback) {
        if (this.elements.testBtn) {
            this.elements.testBtn.addEventListener('click', callback);
        }
    }
    
    bindPauseButton(callback) {
        if (this.elements.pauseBtn) {
            console.log('Binding pause button click event');
            this.elements.pauseBtn.addEventListener('click', (e) => {
                console.log('Pause button clicked');
                callback();
            });
        } else {
            console.error('Pause button not found in DOM');
        }
    }

    getSelectedNoteType() {
        return this.elements.noteType ? this.elements.noteType.value : 'soap';
    }

    // Patient info methods
    getPatientInfo() {
        return {
            firstName: this.elements.firstName ? this.elements.firstName.value.trim() : '',
            lastName: this.elements.lastName ? this.elements.lastName.value.trim() : '',
            dob: this.elements.dob ? this.elements.dob.value : ''
        };
    }

    validatePatientInfo() {
        const info = this.getPatientInfo();
        if (!info.firstName || !info.lastName || !info.dob) {
            this.showError('Please fill in all patient information fields');
            return false;
        }
        return true;
    }

    clearPatientInfo() {
        if (this.elements.firstName) this.elements.firstName.value = '';
        if (this.elements.lastName) this.elements.lastName.value = '';
        if (this.elements.dob) this.elements.dob.value = '';
    }

    // Sidebar methods
    setupSidebarToggle() {
        if (this.elements.sidebarToggle && this.elements.sidebar) {
            this.elements.sidebarToggle.addEventListener('click', () => {
                this.elements.sidebar.classList.toggle('collapsed');
            });
        }
    }
    
    ensureSidebarOpen() {
        if (this.elements.sidebar && this.elements.sidebar.classList.contains('collapsed')) {
            this.elements.sidebar.classList.remove('collapsed');
        }
    }
    
    isSidebarOpen() {
        return this.elements.sidebar && !this.elements.sidebar.classList.contains('collapsed');
    }
    
    scrollToTodaysNotes() {
        const todaysNotesElement = this.elements.notesList?.querySelector('.today-notes');
        if (todaysNotesElement && this.elements.notesList) {
            todaysNotesElement.scrollIntoView({ 
                behavior: 'smooth', 
                block: 'start',
                inline: 'nearest'
            });
        }
    }
    
    // Settings panel methods
    setupSettingsToggle() {
        if (this.elements.settingsToggle && this.elements.settingsPanel) {
            this.elements.settingsToggle.addEventListener('click', () => {
                this.elements.settingsPanel.classList.toggle('collapsed');
            });
        }
    }

    updateNotesList(groupedNotes) {
        if (!this.elements.notesList) return;

        this.elements.notesList.innerHTML = '';
        
        // Update sidebar header with today's notes count
        this.updateSidebarHeader(groupedNotes);

        if (groupedNotes.length === 0) {
            this.elements.notesList.innerHTML = `
                <div style="padding: 20px; text-align: center; color: #666;">
                    <div style="font-size: 2rem; margin-bottom: 10px;">📝</div>
                    <div>No saved notes yet</div>
                    <div style="font-size: 0.9rem; margin-top: 10px; color: #999;">
                        Start recording to create your first note
                    </div>
                </div>
            `;
            return;
        }

        groupedNotes.forEach(group => {
            const dateGroup = document.createElement('div');
            dateGroup.className = 'note-date-group';
            
            // Add special styling for today's notes
            if (group.isToday) {
                dateGroup.classList.add('today-notes');
            }

            const dateHeader = document.createElement('div');
            dateHeader.className = 'note-date-header';
            
            // Add "Today" indicator and note count
            if (group.isToday) {
                dateHeader.innerHTML = `
                    <span class="today-indicator">📅 ${group.date}</span>
                    <span class="note-count">${group.notes.length} note${group.notes.length === 1 ? '' : 's'}</span>
                `;
            } else {
                dateHeader.textContent = group.date;
            }
            
            dateGroup.appendChild(dateHeader);

            group.notes.forEach(note => {
                const noteItem = document.createElement('div');
                noteItem.className = 'note-item';
                noteItem.dataset.noteId = note.id;
                
                // Add special styling for today's note items
                if (group.isToday) {
                    noteItem.classList.add('today-note-item');
                }

                const noteHeader = document.createElement('div');
                noteHeader.className = 'note-item-header';
                noteHeader.textContent = `${note.first_name} ${note.last_name}`;

                const noteDetails = document.createElement('div');
                noteDetails.className = 'note-item-details';
                const dob = new Date(note.dob).toLocaleDateString('en-US', {
                    year: 'numeric',
                    month: 'short',
                    day: 'numeric'
                });
                const time = new Date(note.created_at).toLocaleTimeString('en-US', {
                    hour: '2-digit',
                    minute: '2-digit'
                });
                noteDetails.textContent = `DOB: ${dob} • ${time} • ${note.note_type.toUpperCase()}`;

                noteItem.appendChild(noteHeader);
                noteItem.appendChild(noteDetails);
                dateGroup.appendChild(noteItem);
            });

            this.elements.notesList.appendChild(dateGroup);
        });
    }

    bindNoteClickHandler(callback) {
        if (this.elements.notesList) {
            this.elements.notesList.addEventListener('click', (e) => {
                const noteItem = e.target.closest('.note-item');
                if (noteItem) {
                    // Remove active class from all notes
                    document.querySelectorAll('.note-item').forEach(item => {
                        item.classList.remove('active');
                    });
                    // Add active class to clicked note
                    noteItem.classList.add('active');
                    // Call the callback with note ID
                    callback(noteItem.dataset.noteId);
                }
            });
        }
    }

    // Save button methods
    enableSaveButton(enable = true) {
        if (this.elements.saveBtn) {
            this.elements.saveBtn.disabled = !enable;
        }
    }

    bindSaveButton(callback) {
        if (this.elements.saveBtn) {
            this.elements.saveBtn.addEventListener('click', callback);
        }
    }

    // Display selected note
    displaySelectedNote(note) {
        if (note) {
            this.showTranscript(note.transcript);
            this.showMedicalNote(note.medical_note);
            
            // Update patient info fields
            if (this.elements.firstName) this.elements.firstName.value = note.first_name;
            if (this.elements.lastName) this.elements.lastName.value = note.last_name;
            if (this.elements.dob) this.elements.dob.value = note.dob;
            if (this.elements.noteType) this.elements.noteType.value = note.note_type;
            
            this.updateStatus(`Viewing note from ${new Date(note.created_at).toLocaleString()}`);
        }
    }
    
    // Waveform visualization methods
    startWaveform(stream) {
        if (this.waveformVisualizer) {
            this.waveformVisualizer.startVisualization(stream);
        }
    }
    
    stopWaveform() {
        if (this.waveformVisualizer) {
            this.waveformVisualizer.stopVisualization();
        }
    }
    
    pauseWaveform() {
        if (this.waveformVisualizer) {
            this.waveformVisualizer.pauseVisualization();
        }
    }
    
    resumeWaveform() {
        if (this.waveformVisualizer) {
            this.waveformVisualizer.resumeVisualization();
        }
    }
    
    updateSidebarHeader(groupedNotes) {
        const sidebarHeader = this.elements.sidebar?.querySelector('.sidebar-header h2');
        if (!sidebarHeader) return;
        
        // Find today's notes
        const todayGroup = groupedNotes.find(group => group.isToday);
        const todayCount = todayGroup ? todayGroup.notes.length : 0;
        
        if (todayCount > 0) {
            sidebarHeader.innerHTML = `
                Saved Notes
                <span class="today-badge">${todayCount} today</span>
            `;
        } else {
            const totalCount = groupedNotes.reduce((sum, group) => sum + group.notes.length, 0);
            if (totalCount > 0) {
                sidebarHeader.innerHTML = `
                    Saved Notes
                    <span class="total-badge">${totalCount} total</span>
                `;
            } else {
                sidebarHeader.textContent = 'Saved Notes';
            }
        }
    }
}