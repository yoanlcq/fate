# Si y'a conflits, faut ajouter, à la fin, soit:
# SOIT `--keep-local` (garder les fichiers locaux en cas de conflit)
# SOIT `--keep-remote` (garder les fichiers distants en cas de conflit)

local=./data
folder_id=1PpEbHm5CE1xHqrUOOX21wtIhH9KVo7nW
gdrive_args=${@:2}

case $1 in
    help)
        gdrive help sync download $gdrive_args
        ;;
    pull)
        gdrive sync download $folder_id $local $gdrive_args
        ;;
    push)
        gdrive sync upload $local $folder_id $gdrive_args
        ;;
    *)
        echo "Usage: $0 <pull|push>"
        exit
        ;;
esac

