// Audio Converter Module - Handles WebM to WAV conversion optimized for large files
export class AudioConverter {
    static async webmToWav(webmBlob) {
        try {
            // Use OfflineAudioContext for better performance with large files
            const audioContext = new (window.AudioContext || window.webkitAudioContext)({
                sampleRate: 16000  // Whisper prefers 16kHz
            });

            // Read the WebM blob as array buffer
            const arrayBuffer = await webmBlob.arrayBuffer();
            
            // Decode the audio data
            const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);
            
            // Check if audio has any actual content
            const channelData = audioBuffer.getChannelData(0);
            const maxAmplitude = AudioConverter.getMaxAmplitude(channelData);
            
            if (maxAmplitude < 0.001) {
                console.warn('Audio appears to be silent or very quiet');
            }
            
            // Convert to WAV format using optimized method
            const wavBlob = AudioConverter.audioBufferToWavOptimized(audioBuffer);
            
            // Clean up audio context
            audioContext.close();
            
            return wavBlob;
            
        } catch (error) {
            console.error('Audio conversion failed:', error);
            throw new Error(`Audio conversion failed: ${error.message}`);
        }
    }
    
    // Optimized method to find max amplitude without stack overflow
    static getMaxAmplitude(channelData) {
        let max = 0;
        const chunkSize = 10000; // Process in chunks to avoid stack overflow
        
        for (let i = 0; i < channelData.length; i += chunkSize) {
            const end = Math.min(i + chunkSize, channelData.length);
            for (let j = i; j < end; j++) {
                const abs = Math.abs(channelData[j]);
                if (abs > max) max = abs;
            }
        }
        
        return max;
    }
    
    // Optimized WAV conversion that handles large files efficiently
    static audioBufferToWavOptimized(audioBuffer) {
        const targetSampleRate = 16000;
        const numberOfChannels = 1; // Force mono for speech recognition
        const format = 1; // PCM
        const bitDepth = 16;
        const bytesPerSample = bitDepth / 8;
        const blockAlign = numberOfChannels * bytesPerSample;
        
        // Get channel data and convert to mono if needed
        let samples;
        if (audioBuffer.numberOfChannels === 1) {
            samples = audioBuffer.getChannelData(0);
        } else {
            // Mix stereo to mono efficiently
            const left = audioBuffer.getChannelData(0);
            const right = audioBuffer.getChannelData(1);
            samples = new Float32Array(left.length);
            
            // Process in chunks to avoid stack overflow
            const chunkSize = 10000;
            for (let i = 0; i < left.length; i += chunkSize) {
                const end = Math.min(i + chunkSize, left.length);
                for (let j = i; j < end; j++) {
                    samples[j] = (left[j] + right[j]) / 2;
                }
            }
        }
        
        // Resample if needed
        if (audioBuffer.sampleRate !== targetSampleRate) {
            samples = AudioConverter.resampleAudio(samples, audioBuffer.sampleRate, targetSampleRate);
        }
        
        const length = samples.length;
        const arrayBuffer = new ArrayBuffer(44 + length * bytesPerSample);
        const view = new DataView(arrayBuffer);
        
        // Write WAV header
        AudioConverter.writeWavHeader(view, length, targetSampleRate, numberOfChannels, bitDepth, bytesPerSample, blockAlign);
        
        // Convert and write samples efficiently
        AudioConverter.writeSamplesOptimized(view, samples, 44);
        
        return new Blob([arrayBuffer], { type: 'audio/wav' });
    }
    
    // Efficient WAV header writer
    static writeWavHeader(view, length, sampleRate, numberOfChannels, bitDepth, bytesPerSample, blockAlign) {
        const writeString = (offset, string) => {
            for (let i = 0; i < string.length; i++) {
                view.setUint8(offset + i, string.charCodeAt(i));
            }
        };
        
        writeString(0, 'RIFF');
        view.setUint32(4, 36 + length * bytesPerSample, true);
        writeString(8, 'WAVE');
        writeString(12, 'fmt ');
        view.setUint32(16, 16, true);
        view.setUint16(20, 1, true); // PCM format
        view.setUint16(22, numberOfChannels, true);
        view.setUint32(24, sampleRate, true);
        view.setUint32(28, sampleRate * blockAlign, true);
        view.setUint16(32, blockAlign, true);
        view.setUint16(34, bitDepth, true);
        writeString(36, 'data');
        view.setUint32(40, length * bytesPerSample, true);
    }
    
    // Optimized sample writing to avoid stack overflow
    static writeSamplesOptimized(view, samples, offset) {
        // Find max amplitude for normalization
        let maxAmplitude = 0;
        const chunkSize = 10000;
        
        for (let i = 0; i < samples.length; i += chunkSize) {
            const end = Math.min(i + chunkSize, samples.length);
            for (let j = i; j < end; j++) {
                const abs = Math.abs(samples[j]);
                if (abs > maxAmplitude) maxAmplitude = abs;
            }
        }
        
        const normalizeGain = maxAmplitude > 0 ? Math.min(1, 0.8 / maxAmplitude) : 1;
        
        // Write samples in chunks to avoid stack overflow
        let writeOffset = offset;
        for (let i = 0; i < samples.length; i += chunkSize) {
            const end = Math.min(i + chunkSize, samples.length);
            for (let j = i; j < end; j++) {
                const normalizedSample = samples[j] * normalizeGain;
                const sample = Math.max(-1, Math.min(1, normalizedSample));
                const intSample = sample < 0 ? sample * 0x8000 : sample * 0x7FFF;
                view.setInt16(writeOffset, intSample, true);
                writeOffset += 2;
            }
        }
    }
    
    // Simple resampling for large files
    static resampleAudio(inputSamples, inputSampleRate, outputSampleRate) {
        if (inputSampleRate === outputSampleRate) {
            return inputSamples;
        }
        
        const ratio = inputSampleRate / outputSampleRate;
        const outputLength = Math.floor(inputSamples.length / ratio);
        const outputSamples = new Float32Array(outputLength);
        
        // Simple linear interpolation resampling
        for (let i = 0; i < outputLength; i++) {
            const inputIndex = i * ratio;
            const inputFloor = Math.floor(inputIndex);
            const inputCeil = Math.min(inputFloor + 1, inputSamples.length - 1);
            const fraction = inputIndex - inputFloor;
            
            outputSamples[i] = inputSamples[inputFloor] * (1 - fraction) + 
                              inputSamples[inputCeil] * fraction;
        }
        
        return outputSamples;
    }
}