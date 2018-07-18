# Si y'a conflits, faut ajouter, à la fin, soit:
# SOIT `--keep-local` (garder les fichiers locaux en cas de conflit)
# SOIT `--keep-remote` (garder les fichiers distants en cas de conflit)

folder_id=1_Eol0kmAwp1Bx_Fg_srj51jVlp2u4A3V
gdrive_args=${@:2}

mkdir -p ./art

case $1 in
    help)
        gdrive help sync download $gdrive_args
        ;;
    pull)
        gdrive sync download $folder_id ./art $gdrive_args
        ;;
    push)
        gdrive sync upload ./art $folder_id $gdrive_args
        ;;
    *)
        echo "Usage: $0 <pull|push>"
        exit
        ;;
esac

