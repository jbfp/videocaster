export interface AppResult<T> {
    success: boolean;
    obj: T | null;
    error: string | null;
}

export async function getLocalIpAsync(): Promise<string> {
    return fetch('/ip').then(res => res.json());
}

export interface DirectoryItem {
    isDir: boolean;
    name: string;
    path: string;
}

export interface Directory {
    items: DirectoryItem[];
    parent: DirectoryItem | null;
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

export async function getVideoFrame(
    path: string
): Promise<string> {
    const res = await fetch(`/frame?path=${path}`);
    const blob = await res.blob();
    const reader = new FileReader();
    const promise = new Promise<string>(resolve =>
        reader.addEventListener("load", () =>
            resolve(reader.result as string)));
    reader.readAsDataURL(blob);
    return await promise;
}

export async function shutdown(): Promise<void> {
    await fetch("/shutdown", { method: "POST" });
}
