export function encode(s: string | null) {
    return encodeURIComponent(btoa(s));
}

export function decode(s: string) {
    return atob(decodeURIComponent(s));
}
