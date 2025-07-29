// Waveform Visualizer Module - Shows real-time audio waveform during recording
export class WaveformVisualizer {
    constructor(canvasId) {
        this.canvas = document.getElementById(canvasId);
        this.ctx = this.canvas.getContext('2d');
        this.analyser = null;
        this.dataArray = null;
        this.animationId = null;
        this.isActive = false;
        
        // Waveform settings
        this.barWidth = 3;
        this.barSpacing = 1;
        this.maxBars = Math.floor(this.canvas.width / (this.barWidth + this.barSpacing));
        this.smoothingTimeConstant = 0.8;
        
        // Initialize canvas
        this.setupCanvas();
    }
    
    setupCanvas() {
        // Set canvas size for high DPI displays
        const rect = this.canvas.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;
        
        this.canvas.width = rect.width * dpr;
        this.canvas.height = rect.height * dpr;
        this.ctx.scale(dpr, dpr);
        this.canvas.style.width = rect.width + 'px';
        this.canvas.style.height = rect.height + 'px';
        
        // Clear canvas with background
        this.clearWaveform();
    }
    
    startVisualization(stream) {
        try {
            console.log('Starting waveform visualization');
            
            // Create audio context and analyser
            const audioContext = new (window.AudioContext || window.webkitAudioContext)();
            const source = audioContext.createMediaStreamSource(stream);
            
            this.analyser = audioContext.createAnalyser();
            this.analyser.fftSize = 512;
            this.analyser.smoothingTimeConstant = this.smoothingTimeConstant;
            this.analyser.minDecibels = -90;
            this.analyser.maxDecibels = -10;
            
            source.connect(this.analyser);
            
            // Create data array for frequency data
            const bufferLength = this.analyser.frequencyBinCount;
            this.dataArray = new Uint8Array(bufferLength);
            
            this.isActive = true;
            this.animate();
            
        } catch (error) {
            console.error('Error starting waveform visualization:', error);
            this.showErrorMessage('Waveform visualization unavailable');
        }
    }
    
    stopVisualization() {
        console.log('Stopping waveform visualization');
        this.isActive = false;
        
        if (this.animationId) {
            cancelAnimationFrame(this.animationId);
            this.animationId = null;
        }
        
        this.analyser = null;
        this.dataArray = null;
        this.clearWaveform();
    }
    
    pauseVisualization() {
        this.isActive = false;
        if (this.animationId) {
            cancelAnimationFrame(this.animationId);
            this.animationId = null;
        }
        this.showPausedState();
    }
    
    resumeVisualization() {
        if (this.analyser && this.dataArray) {
            this.isActive = true;
            this.animate();
        }
    }
    
    animate() {
        if (!this.isActive || !this.analyser || !this.dataArray) {
            return;
        }
        
        this.animationId = requestAnimationFrame(() => this.animate());
        
        // Get frequency data
        this.analyser.getByteFrequencyData(this.dataArray);
        
        // Calculate average volume for overall level detection
        const average = this.dataArray.reduce((sum, value) => sum + value, 0) / this.dataArray.length;
        
        this.drawWaveform(this.dataArray, average);
    }
    
    drawWaveform(dataArray, averageLevel) {
        const canvasWidth = this.canvas.width / (window.devicePixelRatio || 1);
        const canvasHeight = this.canvas.height / (window.devicePixelRatio || 1);
        
        // Clear canvas
        this.ctx.fillStyle = 'rgba(255, 255, 255, 0.9)';
        this.ctx.fillRect(0, 0, canvasWidth, canvasHeight);
        
        // Downsample data to fit canvas width
        const step = Math.ceil(dataArray.length / this.maxBars);
        const bars = [];
        
        for (let i = 0; i < this.maxBars; i++) {
            let sum = 0;
            const start = i * step;
            const end = Math.min(start + step, dataArray.length);
            
            for (let j = start; j < end; j++) {
                sum += dataArray[j];
            }
            
            bars.push(sum / (end - start));
        }
        
        // Draw bars
        const barTotalWidth = this.barWidth + this.barSpacing;
        const startX = (canvasWidth - (bars.length * barTotalWidth)) / 2;
        
        for (let i = 0; i < bars.length; i++) {
            const barHeight = (bars[i] / 255) * (canvasHeight - 20);
            const x = startX + (i * barTotalWidth);
            const y = (canvasHeight - barHeight) / 2;
            
            // Color based on intensity
            const intensity = bars[i] / 255;
            const hue = Math.max(0, 120 - (intensity * 120)); // Green to red
            const saturation = Math.min(100, 50 + (intensity * 50));
            const lightness = Math.min(70, 30 + (intensity * 40));
            
            this.ctx.fillStyle = `hsl(${hue}, ${saturation}%, ${lightness}%)`;
            this.ctx.fillRect(x, y, this.barWidth, barHeight);
        }
        
        // Show level indicator
        this.drawLevelIndicator(averageLevel, canvasWidth, canvasHeight);
        
        // Show warning if no signal detected
        if (averageLevel < 5) {
            this.showLowSignalWarning(canvasWidth, canvasHeight);
        }
    }
    
    drawLevelIndicator(level, width, height) {
        const levelBarWidth = 200;
        const levelBarHeight = 8;
        const x = (width - levelBarWidth) / 2;
        const y = height - 25;
        
        // Background
        this.ctx.fillStyle = 'rgba(0, 0, 0, 0.2)';
        this.ctx.fillRect(x, y, levelBarWidth, levelBarHeight);
        
        // Level bar
        const levelWidth = (level / 255) * levelBarWidth;
        const levelColor = level < 30 ? '#ff4757' : level < 100 ? '#ffa502' : '#2ed573';
        this.ctx.fillStyle = levelColor;
        this.ctx.fillRect(x, y, levelWidth, levelBarHeight);
        
        // Border
        this.ctx.strokeStyle = 'rgba(0, 0, 0, 0.3)';
        this.ctx.lineWidth = 1;
        this.ctx.strokeRect(x, y, levelBarWidth, levelBarHeight);
    }
    
    showLowSignalWarning(width, height) {
        this.ctx.fillStyle = 'rgba(255, 71, 87, 0.8)';
        this.ctx.font = '12px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif';
        this.ctx.textAlign = 'center';
        this.ctx.fillText('⚠️ Low signal - check microphone', width / 2, 20);
    }
    
    showPausedState() {
        const canvasWidth = this.canvas.width / (window.devicePixelRatio || 1);
        const canvasHeight = this.canvas.height / (window.devicePixelRatio || 1);
        
        // Clear and show paused state
        this.ctx.fillStyle = 'rgba(255, 255, 255, 0.9)';
        this.ctx.fillRect(0, 0, canvasWidth, canvasHeight);
        
        this.ctx.fillStyle = 'rgba(0, 0, 0, 0.5)';
        this.ctx.font = '16px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif';
        this.ctx.textAlign = 'center';
        this.ctx.fillText('⏸️ Recording Paused', canvasWidth / 2, canvasHeight / 2);
    }
    
    clearWaveform() {
        const canvasWidth = this.canvas.width / (window.devicePixelRatio || 1);
        const canvasHeight = this.canvas.height / (window.devicePixelRatio || 1);
        
        this.ctx.fillStyle = 'rgba(255, 255, 255, 0.9)';
        this.ctx.fillRect(0, 0, canvasWidth, canvasHeight);
        
        // Show inactive state
        this.ctx.fillStyle = 'rgba(0, 0, 0, 0.3)';
        this.ctx.font = '14px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif';
        this.ctx.textAlign = 'center';
        this.ctx.fillText('🎤 Start recording to see waveform', canvasWidth / 2, canvasHeight / 2);
    }
    
    showErrorMessage(message) {
        const canvasWidth = this.canvas.width / (window.devicePixelRatio || 1);
        const canvasHeight = this.canvas.height / (window.devicePixelRatio || 1);
        
        this.ctx.fillStyle = 'rgba(255, 255, 255, 0.9)';
        this.ctx.fillRect(0, 0, canvasWidth, canvasHeight);
        
        this.ctx.fillStyle = 'rgba(255, 71, 87, 0.8)';
        this.ctx.font = '12px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif';
        this.ctx.textAlign = 'center';
        this.ctx.fillText(message, canvasWidth / 2, canvasHeight / 2);
    }
}