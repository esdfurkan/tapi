import { invoke } from '@tauri-apps/api/core';

const isTauri = typeof window !== 'undefined' && (window as any).__TAURI_IPC__ !== undefined;

export const api = {
    async command(cmd: string, args: any = {}) {
        console.log(`[API] Executing command: ${cmd}`, args);
        if (isTauri) {
            try {
                const result = await invoke(cmd, args);
                console.log(`[API] Command '${cmd}' result:`, result);
                return result;
            } catch (e) {
                console.error(`[API] Command '${cmd}' failed:`, e);
                throw e;
            }
        } else {
            // ...
            const endpoint = mapCommandToEndpoint(cmd);
            if (!endpoint) {
                console.warn(`[API] Command '${cmd}' NOT mapped for server mode.`);
                throw new Error(`Command '${cmd}' not supported in Server Mode.`);
            }
            // ...

            const options: RequestInit = {
                method: endpoint.method,
                headers: {
                    'Content-Type': 'application/json',
                },
            };

            if (endpoint.method === 'POST') {
                options.body = JSON.stringify(args);
            }

            const response = await fetch(endpoint.path, options);

            if (!response.ok) {
                throw new Error(`Server Error: ${response.statusText}`);
            }

            // Check content type for JSON
            const contentType = response.headers.get('content-type');
            if (contentType && contentType.includes('application/json')) {
                return await response.json();
            }
            return await response.text();
        }
    }
};

interface EndpointConfig {
    path: string;
    method: 'GET' | 'POST';
}

function mapCommandToEndpoint(cmd: string): EndpointConfig | null {
    switch (cmd) {
        case 'load_settings':
            return { path: '/api/settings/load', method: 'GET' };
        case 'save_settings':
            return { path: '/api/settings/save', method: 'POST' };
        // Add translation commands later if needed, currently only settings are critical for UI init
        // case 'start_cli_translation': ... 
        default:
            console.warn(`Command map missing for: ${cmd}`);
            return null;
    }
}
