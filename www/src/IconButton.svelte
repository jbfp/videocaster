<script lang="ts">
    export let disabled: boolean | undefined = undefined;
    export let icon: string;
    export let title: string | undefined = undefined;
</script>

<button class="icon-btn" on:click {title} {disabled}>
    <i class="material-icons">{icon}</i>
    <slot />
</button>

<style>
    .icon-btn {
        align-items: center;
        background: transparent;
        box-sizing: border-box;
        border: none;
        color: inherit;
        cursor: pointer;
        display: flex;
        padding: 0;
        width: fit-content;
    }

    .icon-btn:disabled {
        color: #666666;
        cursor: initial;
    }

    .icon-btn:not(:disabled):hover {
        color: -webkit-activelink;
    }

    .icon-btn:focus {
        outline: none;
    }
</style>
