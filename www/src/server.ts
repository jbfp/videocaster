export interface AppResult<T> {
    success: boolean;
    obj: T | null;
    error: string | null;
}

export async function getLocalIpAsync(): Promise<string> {
    return fetch('/ip').then(res => res.json());
}

export interface DirectoryEntry {
    isDir: boolean;
    name: string;
    path: string;
}

export interface Directory {
    items: DirectoryEntry[];
    path: string;
}

export async function loadDirectoryAsync(
    path?: string
): Promise<AppResult<Directory>> {
    const query = path?.length > 0 ? `?path=${encodeURIComponent(path)}` : "";
    const url = `/fs${query}`;
    return fetch(url).then(res => res.json());
}

export interface Subtitle {
    name: string;
    url: string;
}

export async function searchSubsByMetadataAsync(
    title: string,
    season?: string,
    episode?: string
): Promise<Subtitle[]> {
    let url = `/subtitles/by-metadata?title=${title}`;

    if (season) {
        url = `${url}&season=${season}`;
    }

    if (episode) {
        url = `${url}&episode=${episode}`;
    }

    return fetch(url).then(res => res.json());
}

export async function searchSubsByPath(
    path: string
): Promise<Subtitle[]> {
    return fetch(`/subtitles/by-path?path=${path}`).then(res => res.json());
}
