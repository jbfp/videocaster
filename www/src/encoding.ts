export function encode(s: string | null) {
    return encodeURIComponent(btoa(encodeURIComponent(s)));
}

export function decode(s: string) {
    return decodeURIComponent(atob(decodeURIComponent(s)));
}
