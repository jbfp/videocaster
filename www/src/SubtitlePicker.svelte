<script lang="ts">
    import { afterUpdate } from "svelte";

    interface Metadata {
        title: string;
        season?: string;
        episode?: string;
    }

    interface Subtitle {
        name: string;
        url: string;
    }

    const regex = /(.+)[sS](\d{1,2})[eE](\d{1,2}).*/;

    export let path: string | null = null;

    let subtitles: Subtitle[];

    async function search(url: string): Promise<Subtitle[]> {
        let res: Response;

        try {
            res = await fetch(url);
        } catch (e) {
            console.error(e);
        }

        return await res.json();
    }

    function searchByPath(path: string) {
        return search(
            "/subtitles/by-path" + `?path=${encodeURIComponent(path)}`
        );
    }

    function searchByMetadata({ title, season, episode }: Metadata) {
        return search(
            "/subtitles/by-metadata" +
                `?title=${encodeURIComponent(title)}` +
                `&season=${season}` +
                `&episode=${episode}`
        );
    }

    function verifyMetadata(path: string): Metadata {
        let title: string, season: string, episode: string;

        const split = path.split("/");
        const fileName = split[split.length - 1];
        const result = regex.exec(fileName);

        console.info("regex test", result);

        if (result) {
            title = result[1].trim();
            season = result[2];
            episode = result[3];

            title = prompt(
                `Is "${title}" the correct title? Note that you may have to add characters such as apostrophes and quotes to accurately reflect the title of the video.`,
                title
            );

            if (!title) {
                // prompt was canceled
                throw new Error("canceled");
            }

            if (season) {
                season = prompt(`Is ${season} the correct season?`, season);
            }

            if (episode) {
                episode = prompt(`Is ${episode} the correct episode?`, episode);
            }
        } else {
            title = fileName;
        }

        return { title, season, episode };
    }

    afterUpdate(async () => {
        if (path === null || subtitles) {
            return;
        }

        subtitles = await searchByPath(path);

        if (subtitles) {
            return;
        }

        let metadata: Metadata;

        try {
            metadata = verifyMetadata(path);
        } catch (e) {
            if (e instanceof Error && e.message === "canceled") {
                return;
            }

            throw e;
        }

        subtitles = await searchByMetadata(metadata);
    });
</script>

{#if subtitles}
    <div>
        <select>
            <option>No subtitles</option>

            {#each subtitles as subtitle}
                <option value={subtitle.url}>{subtitle.name}</option>
            {/each}
        </select>
    </div>
{/if}
