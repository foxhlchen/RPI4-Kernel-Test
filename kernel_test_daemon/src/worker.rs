
use regex::Regex;
//use email_parser::email::Email;
use mailparse::*;
use chrono::prelude::*;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};


mod cfg;

fn main() {
    let text = 
r#"Delivered-To: foxhlchen@gmail.com
Received: by 2002:a05:6520:430d:b029:c8:a3f7:4c3e with SMTP id c13csp3443050lka;
    Mon, 5 Apr 2021 02:13:40 -0700 (PDT)
X-Google-Smtp-Source: ABdhPJxNzWFWJrXLlG8QUDtQmYgxKkHjEVFCDWJpJGdU9dWT8kOK9ELqBcWOu5p8LsschcA2he8h
X-Received: by 2002:a6b:c997:: with SMTP id z145mr18561220iof.36.1617614020295;
    Mon, 05 Apr 2021 02:13:40 -0700 (PDT)
ARC-Seal: i=1; a=rsa-sha256; t=1617614020; cv=none;
    d=google.com; s=arc-20160816;
    b=LRhKRMbivD9hCpbmPA0NKjC2Ig66TN2bKxbWYIKcj9HNai8/7CoX7TthJZdiJTE604
        VNS8xATOJAFlhBGEYGgOxp9BjDGvCgL5HOC7RzK3acDxhISfYSv1MooGXwHmd6qzfzW9
        +cNUXNB2UKIzIhuiJSyuCL7DpvWaYoiJxkKOviRoe3pPaROSBRLuy2o/vwhp24aYgnhh
        t6MYJxGCriws+kmYfYiVeaoGCqWg5kSaGNL0v8Fkhd5RRPgsUiqz+1CiyKVZdQkN1WGD
        jBKJp7McIHcgCsBu6NQqNSu3jTNT27St4yNBptCzSidku6102YA97qL/6TRo9fSyX7Oi
        DQ4g==
ARC-Message-Signature: i=1; a=rsa-sha256; c=relaxed/relaxed; d=google.com; s=arc-20160816;
    h=list-id:precedence:content-transfer-encoding:user-agent
        :mime-version:message-id:date:subject:cc:to:from:dkim-signature;
    bh=+Jyflmk2D08m0/v86yZO5UdllxiET1wXCNCvj9GMUhY=;
    b=MDjyE47kWRfb3cHlUPE6fIi4XNsY7bYdmgtE40WPzoLocp8tWD27IwnMoZks7I6gs3
        ISyNHLdwCvjNpW3g7kFrBsxikdcHvVX/tL5N7IJMKuWgpZQ4UDMfz/Yw4AfKpPzMPOoW
        ty3v7t1XKdVZJfURFc4Jn3VaTDFT5kFle7/bY235PeBPubsC3x5cIFAkYdKj7FhjnpjM
        9e9ounQNCvuzOHkoaTX2QuZFiawDMzZo6aEkZt15+kUQEyAhlAZ5YOro7hTGUo5787cS
        ISlDAxmMKhT+jFao3RQjzhW5tO7vxEmjv+iC2ZZ30ipBg5HFNhriz/nhJw5iFjGgemOt
        IIBA==
ARC-Authentication-Results: i=1; mx.google.com;
    dkim=pass header.i=@linuxfoundation.org header.s=korg header.b=fVqtrAJi;
    spf=pass (google.com: domain of stable-owner@vger.kernel.org designates 23.128.96.18 as permitted sender) smtp.mailfrom=stable-owner@vger.kernel.org;
    dmarc=pass (p=NONE sp=NONE dis=NONE) header.from=linuxfoundation.org
Return-Path: <stable-owner@vger.kernel.org>
Received: from vger.kernel.org (vger.kernel.org. [23.128.96.18])
    by mx.google.com with ESMTP id h15si12803235ila.123.2021.04.05.02.13.26;
    Mon, 05 Apr 2021 02:13:40 -0700 (PDT)
Received-SPF: pass (google.com: domain of stable-owner@vger.kernel.org designates 23.128.96.18 as permitted sender) client-ip=23.128.96.18;
Authentication-Results: mx.google.com;
    dkim=pass header.i=@linuxfoundation.org header.s=korg header.b=fVqtrAJi;
    spf=pass (google.com: domain of stable-owner@vger.kernel.org designates 23.128.96.18 as permitted sender) smtp.mailfrom=stable-owner@vger.kernel.org;
    dmarc=pass (p=NONE sp=NONE dis=NONE) header.from=linuxfoundation.org
Received: (majordomo@vger.kernel.org) by vger.kernel.org via listexpand
    id S239327AbhDEJNP (ORCPT <rfc822;foxhlchen@gmail.com> + 99 others);
    Mon, 5 Apr 2021 05:13:15 -0400
Received: from mail.kernel.org ([198.145.29.99]:60270 "EHLO mail.kernel.org"
    rhost-flags-OK-OK-OK-OK) by vger.kernel.org with ESMTP
    id S239339AbhDEJNA (ORCPT <rfc822;stable@vger.kernel.org>);
    Mon, 5 Apr 2021 05:13:00 -0400
Received: by mail.kernel.org (Postfix) with ESMTPSA id EB529613A1;
    Mon,  5 Apr 2021 09:12:47 +0000 (UTC)
DKIM-Signature: v=1; a=rsa-sha256; c=relaxed/simple; d=linuxfoundation.org;
    s=korg; t=1617613968;
    bh=WfRxY5IdCRC97QW6ab2FxX178m500iD+SxdeDGeck6I=;
    h=From:To:Cc:Subject:Date:From;
    b=fVqtrAJixBOzkkbi5FWgGGTEjw+TNtSdjNc+vzD4+Lfs88FcyOjxFzQr7uz2OPoLw
        0Td8bnKeFI8Flyo+VcfayZkJn+ninfzbfRcFYsIUr+AkVGZPcEHuInapv6S/d43Ni5
        S0Nx6vKjxvE1JDMwyzGr12EDssnnIrp3gC9iMD3g=
From:   Greg Kroah-Hartman <gregkh@linuxfoundation.org>
To:     linux-kernel@vger.kernel.org
Cc:     Greg Kroah-Hartman <gregkh@linuxfoundation.org>,
    torvalds@linux-foundation.org, akpm@linux-foundation.org,
    linux@roeck-us.net, shuah@kernel.org, patches@kernelci.org,
    lkft-triage@lists.linaro.org, pavel@denx.de, jonathanh@nvidia.com,
    f.fainelli@gmail.com, stable@vger.kernel.org
Subject: [PATCH 5.11 000/152] 5.11.12-rc1 review
Date:   Mon,  5 Apr 2021 10:52:29 +0200
Message-Id: <20210405085034.233917714@linuxfoundation.org>
X-Mailer: git-send-email 2.31.1
MIME-Version: 1.0
User-Agent: quilt/0.66
X-stable: review
X-Patchwork-Hint: ignore
X-KernelTest-Patch: http://kernel.org/pub/linux/kernel/v5.x/stable-review/patch-5.11.12-rc1.gz
X-KernelTest-Tree: git://git.kernel.org/pub/scm/linux/kernel/git/stable/linux-stable-rc.git
X-KernelTest-Branch: linux-5.11.y
X-KernelTest-Patches: git://git.kernel.org/pub/scm/linux/kernel/git/stable/stable-queue.git
X-KernelTest-Version: 5.11.12-rc1
X-KernelTest-Deadline: 2021-04-07T08:50+00:00
Content-Type: text/plain; charset=UTF-8
Content-Transfer-Encoding: 8bit
Precedence: bulk
List-ID: <stable.vger.kernel.org>
X-Mailing-List: stable@vger.kernel.org

This is the start of the stable review cycle for the 5.11.12 release.
There are 152 patches in this series, all will be posted as a response
to this one.  If anyone has any issues with these being applied, please
let me know.

Responses should be made by Wed, 07 Apr 2021 08:50:09 +0000.
Anything received after that time might be too late.

The whole patch series can be found in one patch at:
https://www.kernel.org/pub/linux/kernel/v5.x/stable-review/patch-5.11.12-rc1.gz
or in the git tree and branch at:
git://git.kernel.org/pub/scm/linux/kernel/git/stable/linux-stable-rc.git linux-5.11.y
and the diffstat can be found below.

thanks,

greg k-h

-------------
Pseudo-Shortlog of commits:

Greg Kroah-Hartman <gregkh@linuxfoundation.org>
Linux 5.11.12-rc1

David S. Miller <davem@davemloft.net>
Revert "net: bonding: fix error return code of bond_neigh_init()"

Jens Axboe <axboe@kernel.dk>
Revert "kernel: freezer should treat PF_IO_WORKER like PF_KTHREAD for freezing"

Pavel Begunkov <asml.silence@gmail.com>
io_uring: do ctx sqd ejection in a clear context

Ben Dooks <ben.dooks@codethink.co.uk>
riscv: evaluate put_user() arg before enabling user access

Du Cheng <ducheng2@gmail.com>
drivers: video: fbcon: fix NULL dereference in fbcon_cursor()

Ahmad Fatoum <a.fatoum@pengutronix.de>
driver core: clear deferred probe reason on probe retry

Atul Gopinathan <atulgopinathan@gmail.com>
staging: rtl8192e: Change state information from u16 to u8

Atul Gopinathan <atulgopinathan@gmail.com>
staging: rtl8192e: Fix incorrect source in memcpy()

Roja Rani Yarubandi <rojay@codeaurora.org>
soc: qcom-geni-se: Cleanup the code to remove proxy votes

Wesley Cheng <wcheng@codeaurora.org>
usb: dwc3: gadget: Clear DEP flags after stop transfers in ep disable

Shawn Guo <shawn.guo@linaro.org>
usb: dwc3: qcom: skip interconnect init for ACPI probe

Artur Petrosyan <Arthur.Petrosyan@synopsys.com>
usb: dwc2: Prevent core suspend when port connection flag is 0

Artur Petrosyan <Arthur.Petrosyan@synopsys.com>
usb: dwc2: Fix HPRT0.PrtSusp bit setting for HiKey 960 board.

Tong Zhang <ztong0001@gmail.com>
usb: gadget: udc: amd5536udc_pci fix null-ptr-dereference

Johan Hovold <johan@kernel.org>
USB: cdc-acm: fix use-after-free after probe failure

Johan Hovold <johan@kernel.org>
USB: cdc-acm: fix double free on probe failure

Oliver Neukum <oneukum@suse.com>
USB: cdc-acm: downgrade message to debug

Oliver Neukum <oneukum@suse.com>
USB: cdc-acm: untangle a circular dependency between callback and softint

Oliver Neukum <oneukum@suse.com>
cdc-acm: fix BREAK rx code path adding necessary calls

Chunfeng Yun <chunfeng.yun@mediatek.com>
usb: xhci-mtk: fix broken streams issue on 0.96 xHCI

Tony Lindgren <tony@atomide.com>
usb: musb: Fix suspend with devices connected for a64

Vincent Palatin <vpalatin@chromium.org>
USB: quirks: ignore remote wake-up on Fibocom L850-GL LTE modem

Shuah Khan <skhan@linuxfoundation.org>
usbip: vhci_hcd fix shift out-of-bounds in vhci_hub_control()

Zheyu Ma <zheyuma97@gmail.com>
firewire: nosy: Fix a use-after-free bug in nosy_ioctl()

Aneesh Kumar K.V <aneesh.kumar@linux.ibm.com>
powerpc/mm/book3s64: Use the correct storage key value when calling H_PROTECT

Lv Yunlong <lyl2019@mail.ustc.edu.cn>
video: hyperv_fb: Fix a double free in hvfb_probe

Andy Shevchenko <andriy.shevchenko@linux.intel.com>
usb: dwc3: pci: Enable dis_uX_susphy_quirk for Intel Merrifield

Nathan Lynch <nathanl@linux.ibm.com>
powerpc/pseries/mobility: handle premature return from H_JOIN

Nathan Lynch <nathanl@linux.ibm.com>
powerpc/pseries/mobility: use struct for shared state

Richard Gong <richard.gong@intel.com>
firmware: stratix10-svc: reset COMMAND_RECONFIG_FLAG_PARTIAL to 0

Dinghao Liu <dinghao.liu@zju.edu.cn>
extcon: Fix error handling in extcon_dev_register

Krzysztof Kozlowski <krzk@kernel.org>
extcon: Add stubs for extcon_register_notifier_all() functions

Sean Christopherson <seanjc@google.com>
KVM: x86/mmu: Ensure TLBs are flushed for TDP MMU during NX zapping

Paolo Bonzini <pbonzini@redhat.com>
KVM: x86: compile out TDP MMU on 32-bit systems

Ben Gardon <bgardon@google.com>
KVM: x86/mmu: Use atomic ops to set SPTEs in TDP MMU map

Ben Gardon <bgardon@google.com>
KVM: x86/mmu: Factor out functions to add/remove TDP MMU pages

Ben Gardon <bgardon@google.com>
KVM: x86/mmu: Fix braces in kvm_recover_nx_lpages

Ben Gardon <bgardon@google.com>
KVM: x86/mmu: Don't redundantly clear TDP MMU pt memory

Ben Gardon <bgardon@google.com>
KVM: x86/mmu: Add comment on __tdp_mmu_set_spte

Sean Christopherson <seanjc@google.com>
KVM: x86/mmu: Ensure TLBs are flushed when yielding during GFN range zap

Ben Gardon <bgardon@google.com>
KVM: x86/mmu: Protect TDP MMU page table memory with RCU

Ben Gardon <bgardon@google.com>
KVM: x86/mmu: Factor out handling of removed page tables

Ben Gardon <bgardon@google.com>
KVM: x86/mmu: Add lockdep when setting a TDP MMU SPTE

Ben Gardon <bgardon@google.com>
KVM: x86/mmu: Yield in TDU MMU iter even if no SPTES changed

Ben Gardon <bgardon@google.com>
KVM: x86/mmu: Ensure forward progress when yielding in TDP MMU iter

Ben Gardon <bgardon@google.com>
KVM: x86/mmu: Rename goal_gfn to next_last_level_gfn

Ben Gardon <bgardon@google.com>
KVM: x86/mmu: Merge flush and non-flush tdp_mmu_iter_cond_resched

Ben Gardon <bgardon@google.com>
KVM: x86/mmu: change TDP MMU yield function returns to match cond_resched

Arnd Bergmann <arnd@arndb.de>
pinctrl: qcom: fix unintentional string concatenation

Jonathan Marek <jonathan@marek.ca>
pinctrl: qcom: lpass lpi: use default pullup/strength values

Rajendra Nayak <rnayak@codeaurora.org>
pinctrl: qcom: sc7280: Fix SDC1_RCLK configurations

Rajendra Nayak <rnayak@codeaurora.org>
pinctrl: qcom: sc7280: Fix SDC_QDSD_PINGROUP and UFS_RESET offsets

Wang Panzhenzhuan <randy.wang@rock-chips.com>
pinctrl: rockchip: fix restore error in resume

Lars Povlsen <lars.povlsen@microchip.com>
pinctrl: microchip-sgpio: Fix wrong register offset for IRQ trigger

Jason Gunthorpe <jgg@ziepe.ca>
vfio/nvlink: Add missing SPAPR_TCE_IOMMU depends

Thierry Reding <treding@nvidia.com>
drm/tegra: sor: Grab runtime PM reference across reset

Thierry Reding <treding@nvidia.com>
drm/tegra: dc: Restore coupling of display controllers

Pan Bian <bianpan2016@163.com>
drm/imx: fix memory leak when fails to init

Tetsuo Handa <penguin-kernel@i-love.sakura.ne.jp>
reiserfs: update reiserfs_xattrs_initialized() condition

Xℹ Ruoyao <xry111@mengyan1223.wang>
drm/amdgpu: check alignment on CPU page for bo map

Huacai Chen <chenhuacai@kernel.org>
drm/amdgpu: Set a suitable dev_info.gart_page_size

Nirmoy Das <nirmoy.das@amd.com>
drm/amdgpu: fix offset calculation in amdgpu_vm_bo_clear_mappings()

Alex Deucher <alexander.deucher@amd.com>
drm/amdgpu/vangogh: don't check for dpm in is_dpm_running when in suspend

Evan Quan <evan.quan@amd.com>
drm/amd/pm: no need to force MCLK to highest when no display connected

Qu Huang <jinsdb@126.com>
drm/amdkfd: dqm fence memory corruption

Ilya Lipnitskiy <ilya.lipnitskiy@gmail.com>
mm: fix race by making init_zero_pfn() early_initcall

Christian König <christian.koenig@amd.com>
drm/ttm: make ttm_bo_unpin more defensive

Heiko Carstens <hca@linux.ibm.com>
s390/vdso: fix tod_steering_delta type

Heiko Carstens <hca@linux.ibm.com>
s390/vdso: copy tod_steering_delta value to vdso_data page

Steven Rostedt (VMware) <rostedt@goodmis.org>
tracing: Fix stack trace event size

Adrian Hunter <adrian.hunter@intel.com>
PM: runtime: Fix ordering in pm_runtime_get_suppliers()

Adrian Hunter <adrian.hunter@intel.com>
PM: runtime: Fix race getting/putting suppliers at probe

Paolo Bonzini <pbonzini@redhat.com>
KVM: SVM: ensure that EFER.SVME is set when running nested guest or on nested vmexit

Paolo Bonzini <pbonzini@redhat.com>
KVM: SVM: load control fields from VMCB12 before checking them

Max Filippov <jcmvbkbc@gmail.com>
xtensa: move coprocessor_flush to the .text section

Max Filippov <jcmvbkbc@gmail.com>
xtensa: fix uaccess-related livelock in do_page_fault

Jeremy Szu <jeremy.szu@canonical.com>
ALSA: hda/realtek: fix mute/micmute LEDs for HP 640 G8

Hui Wang <hui.wang@canonical.com>
ALSA: hda/realtek: call alc_update_headset_mode() in hp_automute_hook

Hui Wang <hui.wang@canonical.com>
ALSA: hda/realtek: fix a determine_headset_type issue for a Dell AIO

Takashi Iwai <tiwai@suse.de>
ALSA: hda: Add missing sanity checks in PM prepare/complete callbacks

Takashi Iwai <tiwai@suse.de>
ALSA: hda: Re-add dropped snd_poewr_change_state() calls

Ikjoon Jang <ikjn@chromium.org>
ALSA: usb-audio: Apply sample rate quirk to Logitech Connect

Hans de Goede <hdegoede@redhat.com>
ACPI: scan: Fix _STA getting called on devices with unmet dependencies

Vitaly Kuznetsov <vkuznets@redhat.com>
ACPI: processor: Fix CPU0 wakeup in acpi_idle_play_dead()

Rafael J. Wysocki <rafael.j.wysocki@intel.com>
ACPI: tables: x86: Reserve memory occupied by ACPI tables

Jesper Dangaard Brouer <brouer@redhat.com>
bpf: Remove MTU check in __bpf_skb_max_len

Jisheng Zhang <Jisheng.Zhang@synaptics.com>
net: 9p: advance iov on empty read

Tong Zhang <ztong0001@gmail.com>
net: wan/lmc: unregister device when no matching device is found

Alex Elder <elder@linaro.org>
net: ipa: fix register write command validation

Alex Elder <elder@linaro.org>
net: ipa: use a separate pointer for adjusted GSI memory

Alex Elder <elder@linaro.org>
net: ipa: remove two unused register definitions

Doug Brown <doug@schmorgal.com>
appletalk: Fix skb allocation size in loopback case

Nathan Rossi <nathan.rossi@digi.com>
net: ethernet: aquantia: Handle error cleanup of start on open

Shuah Khan <skhan@linuxfoundation.org>
ath10k: hold RCU lock when calling ieee80211_find_sta_by_ifaddr()

Johannes Berg <johannes.berg@intel.com>
iwlwifi: pcie: don't disable interrupts for reg_lock

Ido Schimmel <idosch@nvidia.com>
netdevsim: dev: Initialize FIB module after debugfs

Guo-Feng Fan <vincent_fann@realtek.com>
rtw88: coex: 8821c: correct antenna switch function

Wen Gong <wgong@codeaurora.org>
ath11k: add ieee80211_unregister_hw to avoid kernel crash caused by NULL pointer

Luca Pesce <luca.pesce@vimar.com>
brcmfmac: clear EAP/association status bits on linkdown events

Sasha Levin <sashal@kernel.org>
can: tcan4x5x: fix max register value

Dan Carpenter <dan.carpenter@oracle.com>
mptcp: fix bit MPTCP_PUSH_PENDING tests

Jia-Ju Bai <baijiaju1990@gmail.com>
net: bonding: fix error return code of bond_neigh_init()

Paolo Abeni <pabeni@redhat.com>
mptcp: fix race in release_cb

Oleksij Rempel <linux@rempel-privat.de>
net: introduce CAN specific pointer in the struct net_device

Marc Kleine-Budde <mkl@pengutronix.de>
can: dev: move driver related infrastructure into separate subdir

Florian Westphal <fw@strlen.de>
mptcp: provide subflow aware release function

Paolo Abeni <pabeni@redhat.com>
mptcp: fix DATA_FIN processing for orphaned sockets

Davide Caratti <dcaratti@redhat.com>
flow_dissector: fix TTL and TOS dissection on IPv4 fragments

Paolo Abeni <pabeni@redhat.com>
mptcp: add a missing retransmission timer scheduling

Paolo Abeni <pabeni@redhat.com>
mptcp: init mptcp request socket earlier

Paolo Abeni <pabeni@redhat.com>
mptcp: fix poll after shutdown

Paolo Abeni <pabeni@redhat.com>
mptcp: deliver ssk errors to msk

Sasha Levin <sashal@kernel.org>
net: mvpp2: fix interrupt mask/unmask skip condition

Stefan Metzmacher <metze@samba.org>
io_uring: call req_set_fail_links() on short send[msg]()/recv[msg]() with MSG_WAITALL

zhangyi (F) <yi.zhang@huawei.com>
ext4: do not iput inode under running transaction in ext4_rename()

Peter Zijlstra <peterz@infradead.org>
static_call: Align static_call_is_init() patching condition

Tobias Klausmann <tobias.klausmann@freenet.de>
nouveau: Skip unvailable ttm page entries

Josef Bacik <josef@toxicpanda.com>
Revert "PM: ACPI: reboot: Use S5 for reboot"

Stefan Metzmacher <metze@samba.org>
io_uring: imply MSG_NOSIGNAL for send[msg]()/recv[msg]() calls

Elad Grupi <elad.grupi@dell.com>
nvmet-tcp: fix kmap leak when data digest in use

Waiman Long <longman@redhat.com>
locking/ww_mutex: Fix acquire/release imbalance in ww_acquire_init()/ww_acquire_fini()

Waiman Long <longman@redhat.com>
locking/ww_mutex: Simplify use_ww_ctx & ww_ctx handling

Manaf Meethalavalappu Pallikunhi <manafm@codeaurora.org>
thermal/core: Add NULL pointer check before using cooling device stats

Bard Liao <yung-chuan.liao@linux.intel.com>
ASoC: rt711: add snd_soc_component remove callback

Sameer Pujar <spujar@nvidia.com>
ASoC: rt5659: Update MCLK rate in set_sysclk()

Tong Zhang <ztong0001@gmail.com>
staging: comedi: cb_pcidas64: fix request_irq() warn

Tong Zhang <ztong0001@gmail.com>
staging: comedi: cb_pcidas: fix request_irq() warn

Alexey Dobriyan <adobriyan@gmail.com>
scsi: qla2xxx: Fix broken #endif placement

Lv Yunlong <lyl2019@mail.ustc.edu.cn>
scsi: st: Fix a use after free in st_open()

Pavel Begunkov <asml.silence@gmail.com>
io_uring: halt SQO submission on ctx exit

Pavel Begunkov <asml.silence@gmail.com>
io_uring: fix ->flags races by linked timeouts

Laurent Vivier <lvivier@redhat.com>
vhost: Fix vhost_vq_reset()

Jens Axboe <axboe@kernel.dk>
kernel: freezer should treat PF_IO_WORKER like PF_KTHREAD for freezing

Jiaxin Yu <jiaxin.yu@mediatek.com>
ASoC: mediatek: mt8192: fix tdm out data is valid on rising edge

Olga Kornievskaia <kolga@netapp.com>
NFSD: fix error handling in NFSv4.0 callbacks

Lucas Tanure <tanureal@opensource.cirrus.com>
ASoC: cs42l42: Always wait at least 3ms after reset

Lucas Tanure <tanureal@opensource.cirrus.com>
ASoC: cs42l42: Fix mixer volume control

Lucas Tanure <tanureal@opensource.cirrus.com>
ASoC: cs42l42: Fix channel width support

Lucas Tanure <tanureal@opensource.cirrus.com>
ASoC: cs42l42: Fix Bitclock polarity inversion

Jon Hunter <jonathanh@nvidia.com>
ASoC: soc-core: Prevent warning if no DMI table is present

Hans de Goede <hdegoede@redhat.com>
ASoC: es8316: Simplify adc_pga_gain_tlv table

Benjamin Rood <benjaminjrood@gmail.com>
ASoC: sgtl5000: set DAP_AVC_CTRL register to correct default value on probe

Hans de Goede <hdegoede@redhat.com>
ASoC: rt5651: Fix dac- and adc- vol-tlv values being off by a factor of 10

Hans de Goede <hdegoede@redhat.com>
ASoC: rt5640: Fix dac- and adc- vol-tlv values being off by a factor of 10

Jack Yu <jack.yu@realtek.com>
ASoC: rt1015: fix i2c communication error

Ritesh Harjani <riteshh@linux.ibm.com>
iomap: Fix negative assignment to unsigned sis->pages in iomap_swapfile_activate

J. Bruce Fields <bfields@redhat.com>
rpc: fix NULL dereference on kmalloc failure

Julian Braha <julianbraha@gmail.com>
fs: nfsd: fix kconfig dependency warning for NFSD_V4

Zhaolong Zhang <zhangzl2013@126.com>
ext4: fix bh ref count on error paths

Eric Whitney <enwlinux@gmail.com>
ext4: shrink race window in ext4_should_retry_alloc()

Vivek Goyal <vgoyal@redhat.com>
virtiofs: Fail dax mount if device does not support it

Pavel Tatashin <pasha.tatashin@soleen.com>
arm64: mm: correct the inside linear map range during hotplug check


-------------

Diffstat:

Documentation/virt/kvm/locking.rst                 |   9 +-
Makefile                                           |   4 +-
arch/arm64/mm/mmu.c                                |  20 +-
arch/powerpc/platforms/pseries/lpar.c              |   3 +-
arch/powerpc/platforms/pseries/mobility.c          |  48 ++-
arch/riscv/include/asm/uaccess.h                   |   7 +-
arch/s390/include/asm/vdso/data.h                  |   2 +-
arch/s390/kernel/time.c                            |   1 +
arch/x86/include/asm/kvm_host.h                    |  15 +
arch/x86/include/asm/smp.h                         |   1 +
arch/x86/kernel/acpi/boot.c                        |  25 +-
arch/x86/kernel/setup.c                            |   8 +-
arch/x86/kernel/smpboot.c                          |   2 +-
arch/x86/kvm/Makefile                              |   3 +-
arch/x86/kvm/mmu/mmu.c                             |  49 +--
arch/x86/kvm/mmu/mmu_internal.h                    |   5 +
arch/x86/kvm/mmu/tdp_iter.c                        |  46 +--
arch/x86/kvm/mmu/tdp_iter.h                        |  21 +-
arch/x86/kvm/mmu/tdp_mmu.c                         | 448 +++++++++++++++------
arch/x86/kvm/mmu/tdp_mmu.h                         |  32 +-
arch/x86/kvm/svm/nested.c                          |  28 +-
arch/xtensa/kernel/coprocessor.S                   |  64 +--
arch/xtensa/mm/fault.c                             |   5 +-
drivers/acpi/processor_idle.c                      |   7 +
drivers/acpi/scan.c                                |  12 +-
drivers/acpi/tables.c                              |  42 +-
drivers/base/dd.c                                  |   3 +
drivers/base/power/runtime.c                       |  10 +-
drivers/extcon/extcon.c                            |   1 +
drivers/firewire/nosy.c                            |   9 +-
drivers/gpu/drm/amd/amdgpu/amdgpu_kms.c            |   4 +-
drivers/gpu/drm/amd/amdgpu/amdgpu_vm.c             |  10 +-
drivers/gpu/drm/amd/amdkfd/kfd_dbgdev.c            |   2 +-
.../gpu/drm/amd/amdkfd/kfd_device_queue_manager.c  |   6 +-
.../gpu/drm/amd/amdkfd/kfd_device_queue_manager.h  |   2 +-
drivers/gpu/drm/amd/amdkfd/kfd_packet_manager.c    |   2 +-
drivers/gpu/drm/amd/amdkfd/kfd_packet_manager_v9.c |   2 +-
drivers/gpu/drm/amd/amdkfd/kfd_packet_manager_vi.c |   2 +-
drivers/gpu/drm/amd/amdkfd/kfd_priv.h              |   8 +-
.../gpu/drm/amd/pm/powerplay/hwmgr/smu7_hwmgr.c    |   3 +-
drivers/gpu/drm/amd/pm/swsmu/smu11/vangogh_ppt.c   |   5 +
drivers/gpu/drm/imx/imx-drm-core.c                 |   2 +-
drivers/gpu/drm/nouveau/nouveau_bo.c               |   8 +
drivers/gpu/drm/tegra/dc.c                         |  20 +-
drivers/gpu/drm/tegra/sor.c                        |   7 +
drivers/net/can/Makefile                           |   7 +-
drivers/net/can/dev/Makefile                       |   7 +
drivers/net/can/{ => dev}/dev.c                    |   4 +-
drivers/net/can/{ => dev}/rx-offload.c             |   0
drivers/net/can/m_can/tcan4x5x.c                   |   2 +-
drivers/net/can/slcan.c                            |   4 +-
drivers/net/can/vcan.c                             |   2 +-
drivers/net/can/vxcan.c                            |   6 +-
drivers/net/ethernet/aquantia/atlantic/aq_main.c   |   4 +-
drivers/net/ethernet/marvell/mvpp2/mvpp2_main.c    |   4 +-
drivers/net/ipa/gsi.c                              |  28 +-
drivers/net/ipa/gsi.h                              |   5 +-
drivers/net/ipa/gsi_reg.h                          |  31 +-
drivers/net/ipa/ipa_cmd.c                          |  32 +-
drivers/net/netdevsim/dev.c                        |  40 +-
drivers/net/wan/lmc/lmc_main.c                     |   2 +
drivers/net/wireless/ath/ath10k/wmi-tlv.c          |   7 +-
drivers/net/wireless/ath/ath11k/mac.c              |   7 +-
.../broadcom/brcm80211/brcmfmac/cfg80211.c         |   7 +-
drivers/net/wireless/intel/iwlwifi/pcie/trans.c    |  11 +-
drivers/net/wireless/intel/iwlwifi/pcie/tx-gen2.c  |   5 +-
drivers/net/wireless/intel/iwlwifi/pcie/tx.c       |  22 +-
drivers/net/wireless/realtek/rtw88/rtw8821c.c      |  16 +-
drivers/nvme/target/tcp.c                          |   2 +-
drivers/pinctrl/pinctrl-microchip-sgpio.c          |   2 +-
drivers/pinctrl/pinctrl-rockchip.c                 |  13 +-
drivers/pinctrl/qcom/pinctrl-lpass-lpi.c           |   2 +-
drivers/pinctrl/qcom/pinctrl-sc7280.c              |  16 +-
drivers/pinctrl/qcom/pinctrl-sdx55.c               |   2 +-
drivers/scsi/qla2xxx/qla_target.h                  |   2 +-
drivers/scsi/st.c                                  |   2 +-
drivers/soc/qcom/qcom-geni-se.c                    |  74 ----
drivers/staging/comedi/drivers/cb_pcidas.c         |   2 +-
drivers/staging/comedi/drivers/cb_pcidas64.c       |   2 +-
drivers/staging/rtl8192e/rtllib.h                  |   2 +-
drivers/staging/rtl8192e/rtllib_rx.c               |   2 +-
drivers/thermal/thermal_sysfs.c                    |   3 +
drivers/tty/serial/qcom_geni_serial.c              |   7 -
drivers/usb/class/cdc-acm.c                        |  61 ++-
drivers/usb/core/quirks.c                          |   4 +
drivers/usb/dwc2/hcd.c                             |   5 +-
drivers/usb/dwc3/dwc3-pci.c                        |   2 +
drivers/usb/dwc3/dwc3-qcom.c                       |   3 +
drivers/usb/dwc3/gadget.c                          |   8 +-
drivers/usb/gadget/udc/amd5536udc_pci.c            |  10 +-
drivers/usb/host/xhci-mtk.c                        |  10 +-
drivers/usb/musb/musb_core.c                       |  12 +-
drivers/usb/usbip/vhci_hcd.c                       |   2 +
drivers/vfio/pci/Kconfig                           |   2 +-
drivers/vhost/vhost.c                              |   2 +-
drivers/video/fbdev/core/fbcon.c                   |   3 +
drivers/video/fbdev/hyperv_fb.c                    |   3 -
fs/ext4/balloc.c                                   |  38 +-
fs/ext4/ext4.h                                     |   1 +
fs/ext4/inode.c                                    |   6 +-
fs/ext4/namei.c                                    |  18 +-
fs/ext4/super.c                                    |   5 +
fs/ext4/sysfs.c                                    |   7 +
fs/fuse/virtio_fs.c                                |   9 +-
fs/io_uring.c                                      |  41 +-
fs/iomap/swapfile.c                                |  10 +
fs/nfsd/Kconfig                                    |   1 +
fs/nfsd/nfs4callback.c                             |   1 +
fs/reiserfs/xattr.h                                |   2 +-
include/drm/ttm/ttm_bo_api.h                       |   6 +-
include/linux/acpi.h                               |   9 +-
include/linux/can/can-ml.h                         |  12 +
include/linux/extcon.h                             |  23 ++
.../linux/firmware/intel/stratix10-svc-client.h    |   2 +-
include/linux/netdevice.h                          |  34 +-
include/linux/qcom-geni-se.h                       |   2 -
include/linux/ww_mutex.h                           |   5 +-
kernel/locking/mutex.c                             |  25 +-
kernel/reboot.c                                    |   2 -
kernel/static_call.c                               |  14 +-
kernel/trace/trace.c                               |   3 +-
mm/memory.c                                        |   2 +-
net/9p/client.c                                    |   4 -
net/appletalk/ddp.c                                |  33 +-
net/can/af_can.c                                   |  34 +-
net/can/j1939/main.c                               |  22 +-
net/can/j1939/socket.c                             |  13 +-
net/can/proc.c                                     |  19 +-
net/core/filter.c                                  |  12 +-
net/core/flow_dissector.c                          |   6 +-
net/mptcp/options.c                                |   3 +-
net/mptcp/protocol.c                               | 111 ++++-
net/mptcp/protocol.h                               |   4 +
net/mptcp/subflow.c                                |  83 ++--
net/sunrpc/auth_gss/svcauth_gss.c                  |  11 +-
sound/pci/hda/hda_intel.c                          |   8 +
sound/pci/hda/patch_realtek.c                      |   4 +-
sound/soc/codecs/cs42l42.c                         |  74 ++--
sound/soc/codecs/cs42l42.h                         |  13 +-
sound/soc/codecs/es8316.c                          |   9 +-
sound/soc/codecs/rt1015.c                          |   1 +
sound/soc/codecs/rt5640.c                          |   4 +-
sound/soc/codecs/rt5651.c                          |   4 +-
sound/soc/codecs/rt5659.c                          |   5 +
sound/soc/codecs/rt711.c                           |   8 +
sound/soc/codecs/sgtl5000.c                        |   2 +-
sound/soc/mediatek/mt8192/mt8192-dai-tdm.c         |   4 +-
sound/soc/mediatek/mt8192/mt8192-reg.h             |   8 +-
sound/soc/soc-core.c                               |   4 +
sound/usb/quirks.c                                 |   1 +
.../testing/selftests/net/forwarding/tc_flower.sh  |  38 +-
151 files changed, 1525 insertions(+), 821 deletions(-)
    "#;
    let parsed_rs = parse_mail(text.as_bytes());
    if parsed_rs.is_err() {
        let error = parsed_rs.unwrap_err();
        println!("parse mail failed. error: {}", error);

        return;
    }
    let parsed = parsed_rs.unwrap();
    let headers = parsed.get_headers();
    let subject = headers.get_first_header("X-KernelTest-Patch").unwrap();

    println!("{}", subject.get_value());
    
                              //"1996-12-19T16:39:57-08:00";
    let deadline = String::from(format_deadline("2021-04-07T08:50+00:00"));

    let rfc3339 = DateTime::parse_from_rfc3339(&deadline);
    if let Err(error) = rfc3339 {
        println!("error deadline {} {}", &deadline, error);
    }
    let deadline = rfc3339.unwrap();
    let now = Local::now();

    if now > deadline {
        println!("expired task deadline {} now {}", &deadline, &now);
    }

    println!("None");

    let email = Message::builder()
    .from("Fox Chen <foxhlchen@gmail.com>".parse().unwrap())
    .in_reply_to("<606ff53b.1c69fb81.28e33.42d4@mx.google.com>".to_owned())
    .to("Fox Chen <foxhlchen@gmail.com>".parse().unwrap())
    .subject("A Test")
    .body("test!".to_owned())
    .unwrap();

    let cfgmgr = match cfg::ConfigMgr::new() {
        Ok(config) => config,
        Err(e) => panic!("{}", e.to_string())
    };

    let creds = Credentials::new(cfgmgr.get().smtp.username.to_string(), cfgmgr.get().smtp.password.to_string());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(&cfgmgr.get().smtp.domain)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {:?}", e),
    }
}

fn format_deadline(deadline: &str) -> String {
    let mut s = String::new();
    let mut cnt = 0;
    for c in deadline.chars() {
        if c == '+' && cnt == 1 {
            s.push_str(":00");
        }

        if c == ':' {
            cnt += 1;
        }

        s.push(c);
    }

    s
}