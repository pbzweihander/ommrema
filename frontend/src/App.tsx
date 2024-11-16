import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";

function UploadMod() {
  const queryClient = useQueryClient();

  const modFileRef = useRef<HTMLInputElement>(null);
  const [modFile, setModFile] = useState<File | undefined>(undefined);
  const { mutate: uploadModFile, isPending: isUploadModFilePending } =
    useMutation({
      mutationKey: ["uploadModFile"],
      mutationFn: async (file: File) => {
        const resp = await fetch(`/api/mod/${file.name}`, {
          method: "POST",
          body: file,
        });
        if (resp.status === 200 && modFileRef.current != null) {
          modFileRef.current.value = "";
          queryClient.invalidateQueries({ queryKey: ["mods"] });
        }
      },
    });

  return (
    <>
      <h2 className="mb-4 text-lg font-bold">Upload Mod</h2>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (modFile !== undefined) {
            uploadModFile(modFile);
          }
        }}
      >
        <input
          type="file"
          ref={modFileRef}
          className="file-input file-input-bordered mr-2"
          onChange={(e) => {
            setModFile(e.target.files?.[0]);
          }}
        />
        <input
          type="submit"
          className="btn btn-primary"
          value="Upload"
          disabled={isUploadModFilePending}
        />
      </form>
    </>
  );
}

interface Mod {
  name: string;
  lastModified: string;
  size: number;
}

function ModList() {
  const { data: mods } = useQuery({
    queryKey: ["mods"],
    queryFn: async () => {
      const resp = await fetch("/api/mod");
      if (resp.status === 200) {
        return (await resp.json()) as Mod[];
      }
    },
  });

  return (
    <>
      <h2 className="mb-4 text-lg font-bold">Mod List</h2>
      <ul className="menu">
        {mods?.map((mod) => {
          const size = Math.floor(mod.size / 1024 / 1024);
          return (
            <li key={mod.name}>
              <span>
                <span className="text-lg">{mod.name}</span>
                <span className="grow" />
                <span className="flex flex-col items-end">
                  <span>{size}MB</span>
                  <span>{new Date(mod.lastModified).toLocaleString()}</span>
                </span>
              </span>
            </li>
          );
        })}
      </ul>
    </>
  );
}

function Advanced() {
  const { mutate: reindex, isPending: isReindexPending } = useMutation({
    mutationKey: ["reindex"],
    mutationFn: async () => {
      await fetch(`/api/reindex`, {
        method: "POST",
      });
    },
  });

  return (
    <>
      <h2 className="mb-4 text-lg font-bold">Advanced</h2>
      <button
        className="btn btn-error"
        onClick={() => {
          reindex();
        }}
        disabled={isReindexPending}
      >
        Reindex
      </button>
    </>
  );
}

export default function App() {
  const { data: username } = useQuery({
    queryKey: ["username"],
    queryFn: async () => {
      const resp = await fetch("/api/username");
      if (resp.status === 200) {
        return await resp.text();
      }
    },
  });
  const isLoggedIn = username !== undefined;

  return (
    <>
      <div className="navbar bg-base-200 px-5">
        <h1 className="text-lg font-bold">Ommrema</h1>
        <span className="grow" />
        {isLoggedIn ? (
          <span className="mx-2">{username}</span>
        ) : (
          <a className="btn mx-2 bg-base-300" href="/auth">
            Login
          </a>
        )}
      </div>
      <div className="flex w-screen p-2 md:justify-center">
        {isLoggedIn && (
          <div className="py-2">
            <UploadMod />
            <div className="divider" />
            <ModList />
            <div className="divider" />
            <Advanced />
          </div>
        )}
      </div>
    </>
  );
}
