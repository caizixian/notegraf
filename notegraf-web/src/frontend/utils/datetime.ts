export function showAgo(t: string) {
    const date = Date.parse(t);
    const delta = Date.now() - date; // milliseconds
    const seconds = Math.round(delta / 1000);
    const rtf = new Intl.RelativeTimeFormat();
    if (seconds < 60) {
        return rtf.format(-seconds, "second");
    }
    const minutes = Math.round(seconds / 60);
    if (minutes < 60) {
        return rtf.format(-minutes, "minute");
    }
    const hours = Math.round(minutes / 60);
    if (hours < 24) {
        return rtf.format(-hours, "hour");
    }
    const days = Math.round(hours / 24);
    return rtf.format(-days, "day");
}