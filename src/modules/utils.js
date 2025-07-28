// Utils Module - Shared utilities and Tauri API helpers

// Check for Tauri APIs
export const hasTauriAPIs = typeof window.__TAURI__ !== 'undefined';

// Tauri API references
export let invoke, writeFile, appLocalDataDir, listen;

if (hasTauriAPIs) {
    invoke = window.__TAURI__.core.invoke;
    writeFile = window.__TAURI__.fs.writeFile;
    appLocalDataDir = window.__TAURI__.path.appLocalDataDir;
    listen = window.__TAURI__.event.listen;
} else {
    console.warn('Tauri APIs not available - running in standalone mode');
}

// Utility functions
export const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

export const formatFileSize = (bytes) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

export const formatDuration = (milliseconds) => {
    const minutes = Math.floor(milliseconds / 60000);
    const seconds = Math.floor((milliseconds % 60000) / 1000);
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
};

export const debounce = (func, wait) => {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
};

export const validateAudioFormat = (fileName) => {
    const supportedFormats = ['.wav', '.mp3', '.flac', '.ogg', '.webm'];
    const extension = fileName.toLowerCase().substring(fileName.lastIndexOf('.'));
    return supportedFormats.includes(extension);
};

export const createDownloadLink = (blob, filename) => {
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    return link;
};

// Error handling utilities
export class AppError extends Error {
    constructor(message, type = 'general') {
        super(message);
        this.name = 'AppError';
        this.type = type;
    }
}

export const handleError = (error, context = '') => {
    console.error(`Error in ${context}:`, error);
    
    if (error instanceof AppError) {
        return error.message;
    }
    
    // Provide user-friendly error messages
    if (error.message.includes('Permission denied')) {
        return 'Permission denied. Please check microphone access.';
    }
    
    if (error.message.includes('NotFound')) {
        return 'Required file or device not found.';
    }
    
    if (error.message.includes('AbortError')) {
        return 'Operation was cancelled.';
    }
    
    return error.message || 'An unexpected error occurred.';
};