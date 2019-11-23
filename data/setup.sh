#!/usr/bin/env bash
# This the starting point for my basic setup script, it still needs
# a lot of work, but should get into a minimal working state on fresh install
# To use it enter
# `sh -c "$(curl -fsSL https://gist.githubusercontent.com/stevepentland/<PATH TO RAW SCRIPT>/setup.sh)"`
BASE_PACKAGES=(
	"zsh"
	"git"
	"exa"
	"ripgrep"
	"restic"
	"neovim"
	"snapd"
	"xclip"
	"curl"
	"tmux"
	"tlp"
	"flameshot"
	"rxvt-unicode"
)
SNAPD_PACKAGES=("bitwarden" "bw")
CLASSIC_SNAPD_PACKAGES=("code-insiders")
NERD_FONTS=(
	"Ubuntu Mono Nerd Font Complete.ttf:UbuntuMono:Regular"
	"Ubuntu Mono Nerd Font Complete Mono.ttf:UbuntuMono:Regular"
	"Inconsolata Regular Nerd Font Complete.ttf:Inconsolata:"
	"Inconsolata Regular Nerd Font Complete Mono.ttf:Inconsolata:"
	"Sauce Code Pro Nerd Font Complete.ttf:SourceCodePro:Regular"
	"Sauce Code Pro Nerd Font Complete Mono.ttf:SourceCodePro:Regular"
	"Fura Code Regular Nerd Font Complete.ttf:FiraCode:Regular"
	"Fura Code Regular Nerd Font Complete Mono.ttf:FiraCode:Regular"
)

check_snap_symlink() {
	if [ ! -f /snap ]; then
		sudo ln -s /var/lib/snapd/snap /snap
	fi
	sudo systemctl enable snapd && sudo systemctl start snapd
}

install_snaps() {
	sudo snap install "${SNAPD_PACKAGES[@]}"
	for package in "${CLASSIC_SNAPD_PACKAGES[@]}"; do
		sudo snap install "$package" --classic
	done
}

install_chezmoi() {
	curl -sfL https://git.io/chezmoi | sh
	~/bin/chezmoi init https://github.com/stevepentland/dotfiles.git
}

setup_zsh() {
	chsh -s "$(command -v zsh)"
	export ZPLUG_HOME=~/.zplug
	git clone https://github.com/zplug/zplug $ZPLUG_HOME
}

setup_tpm() {
	if [ ! -d ~/.tmux/plugins/tpm ]; then
		git clone https://github.com/tmux-plugins/tpm ~/.tmux/plugins/tpm
		~/.tmux/plugins/tpm/bin/install_plugins
	fi
}

install_nerd_fonts() {
	# If we're on non-arch, we need to install nerd fonts by hand
	mkdir -p ~/.local/share/fonts
	pushd ~/.local/share/fonts || exit
	BASE_URL="https://github.com/ryanoasis/nerd-fonts/raw/master/patched-fonts"
	IFS=:
	for fontspec in "${NERD_FONTS[@]}"; do
		read -ar SPEC <<<"$fontspec"
		# Our values only have spaces, so we can quick and dirty encode it
		ENCODED=${SPEC[0]//\ /%20}
		TARGET="${BASE_URL}/${SPEC[1]}/${SPEC[2]}/complete/${ENCODED}"
		curl -fLo "${SPEC[0]}" "$TARGET"
	done
	unset IFS
	popd || exit
	fc-cache -f -v
}

run_aur_install() {
	sudo pacman -S --noconfirm --needed base-devel
	sudo pacman -S --noconfirm yay
	AUR_PACKAGES=("insync" "google-chrome")
	yay -Sy
	yay -S --noconfirm "${AUR_PACKAGES[@]}"
}

install_rust() {
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
	~/.cargo/bin/rustup component add rustfmt clippy rls rust-analysis rust-src
}

install_nvm() {
	curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.0/install.sh | bash
}

fedora_install() {
	BASE_PACKAGES+=("bat")
	sudo dnf install -y "${BASE_PACKAGES[@]}"
	check_snap_symlink
}

arch_install() {
	BASE_PACKAGES+=("bat" "pcsclite" "ccid" "libu2f-host")
	sudo pacman -Syyu
	sudo pacman -S --noconfirm "${BASE_PACKAGES[@]}"
	check_snap_symlink
	run_aur_install
}

debian_install() {
	# zplug does not like mawk https://github.com/zplug/zplug/wiki/FAQ
	sudo apt-get remove -y mawk
	BASE_PACKAGES+=(
		"gawk"
		"pcscd"
		"scdaemon"
		"pcsc-tools"
	)
	sudo apt-get install -y "${BASE_PACKAGES[@]}"
}

cd "${HOME}" || exit

if type -P apt &>/dev/null; then
	debian_install
elif type -P dnf &>/dev/null; then
	fedora_install
elif type -P pacman &>/dev/null; then
	arch_install
else
	echo "Unknown setup style, needs to be added to the script"
	exit 1
fi

install_snaps
install_chezmoi
setup_zsh
setup_tpm
install_rust
install_nvm
install_nerd_fonts

echo "All base items installed..."
echo "Log out and back in to make snaps available then run:"
echo "'bw login' and export the token as instructed, then run"
echo "'chezmoi init' followed by"
echo "'chezmoi apply' to get setup"
echo "Once applied, open a new terminal and 'zplug install' to get all plugins"
