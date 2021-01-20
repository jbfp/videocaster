<script lang="ts">
    import FilePicker from "./states/FilePicker.svelte";
    import SubtitlesPicker from "./states/SubtitlesPicker.svelte";
    import VideoPlayer from "./states/VideoPlayer.svelte";

    let currentState: "file picker" | "subtitles picker" | "video player" =
        "file picker";

    let filePath: string;
    let subtitlesUrl: string | null;

    function catchFilePicked(e: CustomEvent<string>) {
        filePath = e.detail;
        currentState = "subtitles picker";
    }

    function catchSubtitleUrlSelected(e: CustomEvent<string | null>) {
        subtitlesUrl = e.detail;
        currentState = "video player";
    }

    function catchStop() {
        currentState = "file picker";
        filePath = undefined;
        subtitlesUrl = undefined;
    }
</script>

<header>
    <h1>
        Videocaster
        <google-cast-launcher />
    </h1>
</header>

{#if currentState === "file picker"}
    <FilePicker on:filePicked={catchFilePicked} />
{:else if currentState === "subtitles picker"}
    <SubtitlesPicker
        {filePath}
        on:subtitleUrlSelected={catchSubtitleUrlSelected}
    />
{:else if currentState === "video player"}
    <VideoPlayer {filePath} {subtitlesUrl} on:stop={catchStop} />
{/if}

<footer>
    <em>
        Subtitles provided by <a
            href="https://www.opensubtitles.org"
            target="_blank">OpenSubtitles.org</a
        >
        | Made by <a href="https://github.com/jbfp/" target="_blank">jbfp</a>
        |
        <a href="https://github.com/jbfp/videocaster" target="_blank"
            >Source Code</a
        >
    </em>
</footer>

<style>
    google-cast-launcher {
        display: inline-block;
        width: 24px;
        height: 24px;
    }

    em {
        color: #666666;
        display: inline-block;
        font-size: small;
    }
</style>
