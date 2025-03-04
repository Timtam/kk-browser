import { exit } from "@tauri-apps/plugin-process"

function DbNotFound() {
    return (
        <>
            <h2>Database not found</h2>
            <p>
                The komplete.db3 could not be found on your system. Please exit
                the app and make sure to run Komplete Kontrol standalone first
                to create a database that we can use.
            </p>
            <button onClick={async () => exit(0)}>Exit</button>
        </>
    )
}

export default DbNotFound
