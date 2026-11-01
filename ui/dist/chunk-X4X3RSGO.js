import {
  Badge,
  Button,
  DATE_FORMAT,
  DataTable,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  FieldGrid,
  FormatDate,
  Input,
  PageHeader,
  PageLayout,
  PageSection,
  RouteTabs,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Skeleton,
  customInstance,
  useLink
} from "./chunk-CZL7FYB6.js";
import {
  Identifier,
  cn
} from "./chunk-LDYPVTBN.js";

// src/wallet/WalletLayout.tsx
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Loader2, RefreshCw, Wallet } from "lucide-react";
import { toast } from "sonner";
import { Fragment, jsx, jsxs } from "react/jsx-runtime";
function WalletLayout({
  basePath = "/wallet",
  onboarded: controlledOnboarded,
  onLink: controlledOnLink,
  isLinking: controlledIsLinking,
  children
}) {
  const queryClient = useQueryClient();
  const isControlled = controlledOnboarded !== void 0;
  const { data: linkedRes, isLoading: checkingLink } = useQuery({
    queryKey: ["wallet-is-linked"],
    queryFn: () => customInstance(
      "/wallet/is-linked",
      { method: "GET" }
    ),
    enabled: !isControlled
  });
  const linkMutation = useMutation({
    mutationFn: () => customInstance("/wallet/link", { method: "POST" }),
    onSuccess: async () => {
      toast.success("Wallet linked");
      await queryClient.invalidateQueries({ queryKey: ["wallet-is-linked"] });
      await queryClient.invalidateQueries({ queryKey: ["wallet-info-custom"] });
    },
    onError: (err) => {
      toast.error(err instanceof Error ? err.message : "Failed to link wallet");
    }
  });
  const rawData = linkedRes?.data;
  const autoLinked = linkedRes?.status === 200 && (typeof rawData === "boolean" ? rawData : typeof rawData === "object" && rawData !== null && "linked" in rawData ? Boolean(rawData.linked) : true);
  const ownWalletOnboarded = isControlled ? controlledOnboarded : autoLinked;
  const isLinking = isControlled ? Boolean(controlledIsLinking) : linkMutation.isPending;
  const handleLink = () => {
    if (controlledOnLink) {
      void controlledOnLink();
    } else {
      linkMutation.mutate();
    }
  };
  const cleanBase = basePath.replace(/\/$/, "");
  const tabs = [
    { label: "Info", to: `${cleanBase}/info` },
    { label: "DID", to: `${cleanBase}/did` },
    { label: "Keys", to: `${cleanBase}/keys` },
    { label: "Credentials", to: `${cleanBase}/credentials` },
    { label: "OID4VP", to: `${cleanBase}/oidc4vp` },
    { label: "OID4VCI", to: `${cleanBase}/oidc4vci` }
  ];
  return /* @__PURE__ */ jsxs(PageLayout, { children: [
    /* @__PURE__ */ jsx(
      PageHeader,
      {
        title: "Wallet",
        description: "Decentralized identity: DIDs, keys and verifiable credentials.",
        actions: ownWalletOnboarded && /* @__PURE__ */ jsxs(Button, { onClick: handleLink, isLoading: isLinking, variant: "outline", size: "sm", children: [
          !isLinking && /* @__PURE__ */ jsx(RefreshCw, {}),
          isLinking ? "Relinking\u2026" : "Relink wallet"
        ] })
      }
    ),
    !isControlled && checkingLink ? /* @__PURE__ */ jsx("div", { className: "flex items-center justify-center min-h-[300px]", children: /* @__PURE__ */ jsx(Loader2, { className: "h-6 w-6 animate-spin text-brand-sky" }) }) : !ownWalletOnboarded ? /* @__PURE__ */ jsxs("div", { className: "flex flex-col items-center justify-center min-h-[400px] border border-dashed border-ink/10 rounded-xl bg-ink/5 p-12", children: [
      /* @__PURE__ */ jsx(Wallet, { className: "h-12 w-12 text-muted-foreground/40 mb-4" }),
      /* @__PURE__ */ jsx("p", { className: "text-muted-foreground font-medium text-lg", children: "Wallet not linked" }),
      /* @__PURE__ */ jsx("p", { className: "text-muted-foreground/60 text-sm mt-1 mb-6", children: "Link your wallet to start managing your identity." }),
      /* @__PURE__ */ jsx(Button, { onClick: handleLink, isLoading: isLinking, children: "Link wallet" })
    ] }) : /* @__PURE__ */ jsxs(Fragment, { children: [
      /* @__PURE__ */ jsx(RouteTabs, { tabs }),
      children
    ] })
  ] });
}

// src/wallet/WalletInfoView.tsx
import { useQuery as useQuery2 } from "@tanstack/react-query";
import { jsx as jsx2, jsxs as jsxs2 } from "react/jsx-runtime";
function WalletInfoView({ didTabHref = "/wallet/did" }) {
  const Link = useLink();
  const {
    data: response,
    isLoading,
    error
  } = useQuery2({
    queryKey: ["wallet-info-custom"],
    queryFn: () => customInstance("/wallet/info", { method: "GET" })
  });
  const info = response?.status === 200 ? response.data : null;
  if (isLoading) {
    return /* @__PURE__ */ jsxs2("div", { className: "space-y-8", children: [
      /* @__PURE__ */ jsx2(Skeleton, { className: "h-40 w-full" }),
      /* @__PURE__ */ jsx2(Skeleton, { className: "h-64 w-full" })
    ] });
  }
  if (error || !info) {
    return /* @__PURE__ */ jsx2("div", { className: "text-destructive font-mono text-xs", children: "Error loading wallet info" });
  }
  return /* @__PURE__ */ jsxs2("div", { className: "space-y-12 pb-20", children: [
    /* @__PURE__ */ jsx2(PageSection, { title: "General information", children: /* @__PURE__ */ jsx2(
      FieldGrid,
      {
        columns: 4,
        fields: [
          { label: "Wallet ID", value: /* @__PURE__ */ jsx2("span", { className: "font-mono text-xs", children: info.id }) },
          { label: "Name", value: info.name },
          { label: "Created", value: /* @__PURE__ */ jsx2(FormatDate, { date: info.createdOn || null }) },
          {
            label: "Permission level",
            value: /* @__PURE__ */ jsx2(Badge, { variant: "outline", children: info.permission })
          }
        ]
      }
    ) }),
    /* @__PURE__ */ jsxs2(PageSection, { title: "Associated DIDs", children: [
      /* @__PURE__ */ jsxs2("p", { className: "text-xs text-muted-foreground mb-4", children: [
        "Keys and the default DID are managed in the",
        " ",
        /* @__PURE__ */ jsx2(Link, { to: didTabHref, className: "font-medium text-brand-sky hover:underline", children: "DID" }),
        " ",
        "tab."
      ] }),
      /* @__PURE__ */ jsx2(
        DataTable,
        {
          searchable: false,
          data: info.dids,
          keyExtractor: (d) => d.did,
          columns: [
            {
              header: "Alias",
              accessorKey: "alias",
              cell: (d) => /* @__PURE__ */ jsx2("span", { className: "font-medium", children: d.alias })
            },
            {
              header: "DID",
              accessorKey: "did",
              cell: (d) => /* @__PURE__ */ jsx2(Identifier, { value: d.did, tail: 12 })
            },
            {
              header: "Type",
              accessorKey: "type",
              cell: (d) => /* @__PURE__ */ jsx2(Badge, { variant: "info", className: "font-mono", children: d.type })
            },
            {
              header: "Default",
              accessorKey: "default",
              cell: (d) => /* @__PURE__ */ jsx2(Badge, { variant: d.default ? "default" : "info", children: d.default ? "PRIMARY" : "SECONDARY" })
            },
            {
              header: "# Keys",
              cell: (d) => /* @__PURE__ */ jsx2("span", { className: "text-xs text-muted-foreground", children: d.keys.length })
            }
          ]
        }
      )
    ] })
  ] });
}

// src/wallet/WalletDidView.tsx
import { useState as useState3 } from "react";
import { useMutation as useMutation3, useQuery as useQuery4, useQueryClient as useQueryClient3 } from "@tanstack/react-query";
import {
  Check as Check2,
  ChevronRight,
  Copy,
  FileJson,
  Key,
  Loader2 as Loader23,
  Plus as Plus2,
  Star,
  Trash2 as Trash22
} from "lucide-react";
import { toast as toast3 } from "sonner";

// src/ui/checkbox.tsx
import * as React from "react";
import * as CheckboxPrimitive from "@radix-ui/react-checkbox";
import { Check } from "lucide-react";
import { jsx as jsx3 } from "react/jsx-runtime";
var Checkbox = React.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx3(
  CheckboxPrimitive.Root,
  {
    ref,
    className: cn(
      "peer h-4 w-4 shrink-0 rounded-sm border border-ink/20 bg-background-800 ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-brand-sky/60 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=checked]:border-primary data-[state=checked]:text-ink",
      className
    ),
    ...props,
    children: /* @__PURE__ */ jsx3(
      CheckboxPrimitive.Indicator,
      {
        className: cn("flex items-center justify-center text-current"),
        children: /* @__PURE__ */ jsx3(Check, { className: "h-3 w-3 stroke-[3]" })
      }
    )
  }
));
Checkbox.displayName = CheckboxPrimitive.Root.displayName;

// src/ui/label.tsx
import * as React2 from "react";
import * as LabelPrimitive from "@radix-ui/react-label";
import { cva } from "class-variance-authority";
import { jsx as jsx4 } from "react/jsx-runtime";
var labelVariants = cva(
  "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
);
var Label = React2.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx4(LabelPrimitive.Root, { ref, className: cn(labelVariants(), className), ...props }));
Label.displayName = LabelPrimitive.Root.displayName;

// src/wallet/WalletKeysView.tsx
import { useState as useState2 } from "react";
import { useMutation as useMutation2, useQuery as useQuery3, useQueryClient as useQueryClient2 } from "@tanstack/react-query";
import { Loader2 as Loader22, Plus, Trash2 } from "lucide-react";
import { toast as toast2 } from "sonner";

// src/ui/textarea.tsx
import * as React3 from "react";
import { jsx as jsx5 } from "react/jsx-runtime";
var Textarea = React3.forwardRef(
  ({ className, ...props }, ref) => {
    return /* @__PURE__ */ jsx5(
      "textarea",
      {
        className: cn(
          "flex min-h-[60px] w-full rounded-md border border-ink/15 bg-background-800/90 px-3 py-2 text-xs text-brand-snow shadow-sm placeholder:text-muted-foreground/60 transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-brand-sky/60 focus-visible:border-brand-sky/60 disabled:cursor-not-allowed disabled:opacity-50 [color-scheme:dark]",
          className
        ),
        ref,
        ...props
      }
    );
  }
);
Textarea.displayName = "Textarea";

// src/ui/confirm-dialog.tsx
import { useState } from "react";
import { AlertTriangle } from "lucide-react";
import { jsx as jsx6, jsxs as jsxs3 } from "react/jsx-runtime";
function ConfirmDialog({
  open,
  onOpenChange,
  title,
  description,
  children,
  confirmLabel = "Delete",
  destructive = true,
  onConfirm
}) {
  const [pending, setPending] = useState(false);
  const [error, setError] = useState(null);
  const change = (next) => {
    if (pending) return;
    if (!next) setError(null);
    onOpenChange(next);
  };
  const confirm = async () => {
    setPending(true);
    setError(null);
    try {
      await onConfirm();
      onOpenChange(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setPending(false);
    }
  };
  return /* @__PURE__ */ jsx6(Dialog, { open, onOpenChange: change, children: /* @__PURE__ */ jsxs3(DialogContent, { className: "sm:min-w-[480px] gap-4", children: [
    /* @__PURE__ */ jsxs3(DialogHeader, { children: [
      /* @__PURE__ */ jsxs3(DialogTitle, { className: "flex items-center gap-2", children: [
        destructive && /* @__PURE__ */ jsx6(AlertTriangle, { className: "h-5 w-5 text-danger-600" }),
        title
      ] }),
      description && /* @__PURE__ */ jsx6(DialogDescription, { className: "text-sm", children: description })
    ] }),
    children && /* @__PURE__ */ jsx6("div", { className: "text-sm text-ink/75", children }),
    error && /* @__PURE__ */ jsx6("p", { className: "rounded-md border border-danger-700/40 bg-danger-50 px-3 py-2 text-xs text-danger-700 dark:bg-danger-950/40 dark:text-danger-300", children: error }),
    /* @__PURE__ */ jsxs3(DialogFooter, { children: [
      /* @__PURE__ */ jsx6(Button, { variant: "outline", size: "sm", onClick: () => change(false), disabled: pending, children: "Cancel" }),
      /* @__PURE__ */ jsx6(
        Button,
        {
          variant: destructive ? "destructive" : "default",
          size: "sm",
          onClick: confirm,
          isLoading: pending,
          children: confirmLabel
        }
      )
    ] })
  ] }) });
}

// src/wallet/WalletKeysView.tsx
import { jsx as jsx7, jsxs as jsxs4 } from "react/jsx-runtime";
var WALLET_KEYS_QUERY_KEY = ["/wallet/keys"];
function WalletKeysView() {
  const queryClient = useQueryClient2();
  const [deleting, setDeleting] = useState2(null);
  const {
    data: response,
    isLoading,
    error
  } = useQuery3({
    queryKey: WALLET_KEYS_QUERY_KEY,
    queryFn: () => customInstance("/wallet/keys", { method: "GET" })
  });
  const { data: info } = useQuery3({
    queryKey: ["wallet-info-custom"],
    queryFn: () => customInstance("/wallet/info", { method: "GET" })
  });
  const deleteKey = useMutation2({
    mutationFn: ({ id }) => customInstance(`/wallet/key/${encodeURIComponent(id)}`, { method: "DELETE" })
  });
  const refresh = async () => {
    await queryClient.invalidateQueries({ queryKey: WALLET_KEYS_QUERY_KEY });
    await queryClient.invalidateQueries({ queryKey: ["wallet-keys-list"] });
  };
  const dids = info?.status === 200 ? info.data.dids : [];
  const usedBy = (key) => dids.filter((d) => d.keys.some((k) => k.internal === key.id));
  if (isLoading) {
    return /* @__PURE__ */ jsx7(PageSection, { title: "Keys", children: /* @__PURE__ */ jsx7(Skeleton, { className: "h-64 w-full" }) });
  }
  if (error || response?.status !== 200) {
    return /* @__PURE__ */ jsx7("p", { className: "text-sm text-danger-600", children: "Error loading keys." });
  }
  return /* @__PURE__ */ jsxs4(PageSection, { title: "Keys", action: /* @__PURE__ */ jsx7(ImportKeyDialog, { onImported: refresh }), children: [
    /* @__PURE__ */ jsx7(
      DataTable,
      {
        searchable: false,
        data: response.data,
        keyExtractor: (k) => k.id,
        emptyMessage: "No keys in this wallet",
        emptyDescription: "Import a PEM private key to create DIDs with it.",
        columns: [
          {
            header: "Alias",
            accessorKey: "alias",
            cell: (k) => /* @__PURE__ */ jsx7("span", { className: "font-medium", children: k.alias || "\u2014" })
          },
          { header: "Key ID", cell: (k) => /* @__PURE__ */ jsx7(Identifier, { value: k.id }) },
          { header: "Type", cell: (k) => /* @__PURE__ */ jsx7(Badge, { variant: "info", children: k.kty }) },
          {
            header: "Curve",
            cell: (k) => /* @__PURE__ */ jsx7("span", { className: "font-mono text-xs", children: k.crv ?? "\u2014" })
          },
          {
            header: "Used by",
            cell: (k) => {
              const users = usedBy(k);
              return users.length === 0 ? /* @__PURE__ */ jsx7("span", { className: "text-ink/40 text-xs", children: "No DID" }) : /* @__PURE__ */ jsx7("div", { className: "flex flex-wrap gap-1", children: users.map((d) => /* @__PURE__ */ jsx7(Badge, { variant: "outline", title: d.did, children: d.alias }, d.did)) });
            }
          },
          { header: "Created", cell: (k) => /* @__PURE__ */ jsx7(FormatDate, { date: k.created_at ?? null }) },
          {
            header: "",
            cell: (k) => /* @__PURE__ */ jsx7(
              Button,
              {
                size: "sm",
                variant: "ghost",
                "aria-label": `Delete key ${k.alias || k.id}`,
                disabled: usedBy(k).length > 0,
                title: usedBy(k).length > 0 ? "A DID still uses this key" : "Delete key",
                onClick: () => setDeleting(k),
                children: /* @__PURE__ */ jsx7(Trash2, { className: "h-3.5 w-3.5 text-danger-500" })
              }
            )
          }
        ]
      }
    ),
    /* @__PURE__ */ jsx7(
      ConfirmDialog,
      {
        open: deleting !== null,
        onOpenChange: (open) => !open && setDeleting(null),
        title: `Delete key ${deleting?.alias || deleting?.id}?`,
        description: "The private key is removed from the wallet and cannot be recovered.",
        onConfirm: async () => {
          await deleteKey.mutateAsync({ id: deleting.id });
          toast2.success("Key deleted");
          await refresh();
        }
      }
    )
  ] });
}
var ImportKeyDialog = ({ onImported }) => {
  const [open, setOpen] = useState2(false);
  const [pem, setPem] = useState2("");
  const [alias, setAlias] = useState2("");
  const register = useMutation2({
    mutationFn: (data) => customInstance("/wallet/key", { method: "POST", data })
  });
  const looksLikePem = /-----BEGIN [A-Z ]+-----[\s\S]+-----END [A-Z ]+-----/.test(pem.trim());
  const submit = async () => {
    try {
      const res = await register.mutateAsync({ pem, alias: alias || null });
      if (res.status >= 300) throw new Error("The wallet rejected the key");
      toast2.success("Key imported");
      setOpen(false);
      setPem("");
      setAlias("");
      await onImported();
    } catch (err) {
      toast2.error(err instanceof Error ? err.message : "Failed to import key");
    }
  };
  return /* @__PURE__ */ jsxs4(Dialog, { open, onOpenChange: setOpen, children: [
    /* @__PURE__ */ jsx7(DialogTrigger, { asChild: true, children: /* @__PURE__ */ jsxs4(Button, { size: "sm", children: [
      /* @__PURE__ */ jsx7(Plus, {}),
      "Import key"
    ] }) }),
    /* @__PURE__ */ jsxs4(DialogContent, { className: "max-w-2xl", children: [
      /* @__PURE__ */ jsxs4(DialogHeader, { children: [
        /* @__PURE__ */ jsx7(DialogTitle, { children: "Import a private key" }),
        /* @__PURE__ */ jsx7(DialogDescription, { children: "Paste a PEM private key. Type and curve are read from it." })
      ] }),
      /* @__PURE__ */ jsxs4("div", { className: "space-y-4", children: [
        /* @__PURE__ */ jsxs4("div", { className: "space-y-1.5", children: [
          /* @__PURE__ */ jsx7(Label, { htmlFor: "key-alias", children: "Alias" }),
          /* @__PURE__ */ jsx7(
            Input,
            {
              id: "key-alias",
              placeholder: "e.g. signing-2026",
              value: alias,
              onChange: (e) => setAlias(e.target.value)
            }
          )
        ] }),
        /* @__PURE__ */ jsxs4("div", { className: "space-y-1.5", children: [
          /* @__PURE__ */ jsx7(Label, { htmlFor: "key-pem", children: "PEM" }),
          /* @__PURE__ */ jsx7(
            Textarea,
            {
              id: "key-pem",
              className: "min-h-[200px] font-mono text-xs",
              placeholder: "-----BEGIN PRIVATE KEY-----\n\u2026\n-----END PRIVATE KEY-----",
              value: pem,
              onChange: (e) => setPem(e.target.value)
            }
          ),
          pem.trim() && !looksLikePem && /* @__PURE__ */ jsx7("p", { className: "text-xs text-warn-700 dark:text-warn-300", children: "The PEM needs its BEGIN and END lines." })
        ] })
      ] }),
      /* @__PURE__ */ jsxs4(DialogFooter, { children: [
        /* @__PURE__ */ jsx7(Button, { variant: "outline", onClick: () => setOpen(false), children: "Cancel" }),
        /* @__PURE__ */ jsxs4(Button, { disabled: !looksLikePem || register.isPending, onClick: submit, children: [
          register.isPending ? /* @__PURE__ */ jsx7(Loader22, { className: "animate-spin" }) : /* @__PURE__ */ jsx7(Plus, {}),
          "Import"
        ] })
      ] })
    ] })
  ] });
};

// src/wallet/WalletDidView.tsx
import { Fragment as Fragment2, jsx as jsx8, jsxs as jsxs5 } from "react/jsx-runtime";
var INFO_QUERY_KEY = ["wallet-info-custom"];
function reportError(err, fallback) {
  const msg = err instanceof Error ? err.message : fallback;
  toast3.error(msg);
}
function WalletDidView() {
  const queryClient = useQueryClient3();
  const {
    data: response,
    isLoading,
    error
  } = useQuery4({
    queryKey: INFO_QUERY_KEY,
    queryFn: () => customInstance("/wallet/info", { method: "GET" })
  });
  const invalidate = () => queryClient.invalidateQueries({ queryKey: INFO_QUERY_KEY });
  const setDefaultDid = useMutation3({
    mutationFn: (id) => customInstance(`/wallet/did/${encodeURIComponent(id)}/default`, { method: "POST" }),
    onSuccess: async () => {
      toast3.success("Default DID updated");
      await invalidate();
    },
    onError: (err) => reportError(err, "Failed to set default DID")
  });
  if (isLoading) {
    return /* @__PURE__ */ jsx8(PageSection, { title: "DIDs", children: /* @__PURE__ */ jsx8(Skeleton, { className: "h-64 w-full rounded-2xl" }) });
  }
  if (error || response?.status !== 200) {
    return /* @__PURE__ */ jsx8("div", { className: "text-destructive font-mono text-xs", children: "Error loading DIDs" });
  }
  const dids = response.data.dids;
  return /* @__PURE__ */ jsx8("div", { className: "space-y-8 pb-20", children: /* @__PURE__ */ jsx8(PageSection, { title: "DIDs", action: /* @__PURE__ */ jsx8(NewDidDialog, { onCreated: invalidate }), children: dids.length === 0 ? /* @__PURE__ */ jsxs5("div", { className: "flex flex-col items-center justify-center py-20 text-muted-foreground border border-dashed border-ink/10 rounded-2xl bg-ink/2", children: [
    /* @__PURE__ */ jsx8(Key, { className: "h-12 w-12 opacity-10 mb-4" }),
    /* @__PURE__ */ jsx8("p", { className: "text-sm font-medium", children: "No DIDs registered yet." })
  ] }) : /* @__PURE__ */ jsx8("div", { className: "space-y-4", children: dids.map((d) => /* @__PURE__ */ jsx8(
    DidPanel,
    {
      did: d,
      onSetDefault: () => setDefaultDid.mutate(d.id),
      isSettingDefault: setDefaultDid.isPending && setDefaultDid.variables === d.id,
      onChanged: invalidate
    },
    d.did
  )) }) }) });
}
var DidPanel = ({
  did,
  onSetDefault,
  isSettingDefault,
  onChanged
}) => {
  const [isOpen, setIsOpen] = useState3(false);
  const [newKeyId, setNewKeyId] = useState3("");
  const [copied, setCopied] = useState3(false);
  const isWeb = did.type === "Web";
  const addKey = useMutation3({
    mutationFn: (keyId) => customInstance(`/wallet/did/${encodeURIComponent(did.id)}/key/${encodeURIComponent(keyId)}`, {
      method: "POST"
    }),
    onSuccess: async () => {
      toast3.success("Key added to DID");
      setNewKeyId("");
      await onChanged();
    },
    onError: (err) => reportError(err, "Failed to add key")
  });
  const removeKey = useMutation3({
    mutationFn: (keyId) => customInstance(`/wallet/did/${encodeURIComponent(did.id)}/key/${encodeURIComponent(keyId)}`, {
      method: "DELETE"
    }),
    onSuccess: async () => {
      toast3.success("Key removed from DID");
      await onChanged();
    },
    onError: (err) => reportError(err, "Failed to remove key")
  });
  const setDefaultKey = useMutation3({
    mutationFn: (keyId) => customInstance(
      `/wallet/did/${encodeURIComponent(did.id)}/key/default/${encodeURIComponent(keyId)}`,
      { method: "POST" }
    ),
    onSuccess: async () => {
      toast3.success("Default key updated");
      await onChanged();
    },
    onError: (err) => reportError(err, "Failed to set default key")
  });
  const deleteDid = useMutation3({
    mutationFn: () => customInstance(`/wallet/did/${encodeURIComponent(did.id)}`, { method: "DELETE" }),
    onSuccess: async () => {
      toast3.success("DID deleted");
      await onChanged();
    },
    onError: (err) => reportError(err, "Failed to delete DID")
  });
  const formattedDoc = JSON.stringify(did.did_document, null, 2);
  const handleCopy = (e) => {
    e.stopPropagation();
    navigator.clipboard.writeText(formattedDoc);
    setCopied(true);
    setTimeout(() => setCopied(false), 2e3);
  };
  return /* @__PURE__ */ jsxs5("div", { className: "group border border-ink/10 rounded-xl overflow-hidden bg-ink/[0.02] transition-all hover:bg-ink/[0.04]", children: [
    /* @__PURE__ */ jsxs5("div", { className: "w-full flex items-center justify-between p-4 gap-3", children: [
      /* @__PURE__ */ jsxs5(
        "button",
        {
          type: "button",
          onClick: () => setIsOpen(!isOpen),
          className: "flex items-center gap-3 flex-1 min-w-0 text-left",
          children: [
            /* @__PURE__ */ jsx8(
              "div",
              {
                className: cn(
                  "transition-transform duration-200 shrink-0",
                  isOpen ? "rotate-90 text-primary" : "text-muted-foreground"
                ),
                children: /* @__PURE__ */ jsx8(ChevronRight, { className: "h-5 w-5" })
              }
            ),
            /* @__PURE__ */ jsx8("span", { className: "text-sm font-semibold", children: did.alias }),
            did.default && /* @__PURE__ */ jsx8(Badge, { variant: "default", children: "PRIMARY" }),
            /* @__PURE__ */ jsx8(Badge, { variant: "info", className: "font-mono text-xs", children: did.type }),
            /* @__PURE__ */ jsx8(
              "span",
              {
                className: "text-xs text-muted-foreground/60 font-mono truncate max-w-[320px]",
                title: did.did,
                children: did.did
              }
            )
          ]
        }
      ),
      /* @__PURE__ */ jsxs5("div", { className: "flex items-center gap-2 shrink-0", children: [
        !did.default && /* @__PURE__ */ jsxs5(Button, { size: "sm", variant: "outline", disabled: isSettingDefault, onClick: onSetDefault, children: [
          isSettingDefault ? /* @__PURE__ */ jsx8(Loader23, { className: "h-3 w-3 animate-spin" }) : /* @__PURE__ */ jsx8(Star, { className: "h-3 w-3" }),
          /* @__PURE__ */ jsx8("span", { className: "ml-1", children: "Set default" })
        ] }),
        /* @__PURE__ */ jsx8(
          Button,
          {
            size: "sm",
            variant: "ghost",
            disabled: did.default || deleteDid.isPending,
            onClick: () => deleteDid.mutate(),
            title: did.default ? "Cannot delete the active default DID" : "Delete this DID",
            children: deleteDid.isPending ? /* @__PURE__ */ jsx8(Loader23, { className: "h-3 w-3 animate-spin" }) : /* @__PURE__ */ jsx8(Trash22, { className: "h-3 w-3 text-destructive" })
          }
        )
      ] })
    ] }),
    isOpen && /* @__PURE__ */ jsxs5("div", { className: "p-4 pt-0 space-y-6", children: [
      /* @__PURE__ */ jsxs5("div", { children: [
        /* @__PURE__ */ jsxs5("div", { className: "flex items-center gap-2 mb-3", children: [
          /* @__PURE__ */ jsx8(Key, { className: "h-4 w-4 text-primary" }),
          /* @__PURE__ */ jsx8("h5", { className: "text-xs uppercase tracking-widest font-bold text-foreground/70", children: "Attached Keys" })
        ] }),
        /* @__PURE__ */ jsx8(
          DataTable,
          {
            className: "text-sm",
            data: did.keys,
            keyExtractor: (k) => `${k.internal}-${k.fragment}`,
            hideToolbar: true,
            columns: [
              {
                header: "Fragment",
                accessorKey: "fragment",
                cell: (k) => /* @__PURE__ */ jsxs5("span", { className: "font-mono text-xs", children: [
                  "#",
                  k.fragment
                ] })
              },
              {
                header: "Internal id",
                accessorKey: "internal",
                cell: (k) => /* @__PURE__ */ jsx8("span", { className: "font-mono text-xs text-muted-foreground block max-w-[280px] truncate", children: k.internal })
              },
              {
                header: "Default",
                sortValue: (k) => k.internal === did.default_key?.internal,
                searchable: false,
                cell: (k) => k.internal === did.default_key?.internal ? /* @__PURE__ */ jsx8(Badge, { variant: "success", children: "Default" }) : /* @__PURE__ */ jsx8("span", { className: "text-xs text-muted-foreground/50", children: "\u2014" })
              },
              {
                header: "Actions",
                sortable: false,
                searchable: false,
                cell: (k) => /* @__PURE__ */ jsxs5("div", { className: "flex items-center gap-1", children: [
                  k.internal !== did.default_key?.internal && /* @__PURE__ */ jsx8(
                    Button,
                    {
                      size: "icon",
                      variant: "ghost",
                      className: "h-7 w-7",
                      disabled: !isWeb || setDefaultKey.isPending,
                      onClick: () => setDefaultKey.mutate(k.internal),
                      title: isWeb ? "Set as default signing key" : "did:jwk has a single key \u2014 cannot change default",
                      children: /* @__PURE__ */ jsx8(Star, { className: "h-3 w-3" })
                    }
                  ),
                  /* @__PURE__ */ jsx8(
                    Button,
                    {
                      size: "icon",
                      variant: "ghost",
                      className: "h-7 w-7",
                      disabled: !isWeb || did.keys.length === 1 || removeKey.isPending,
                      onClick: () => removeKey.mutate(k.internal),
                      title: !isWeb ? "did:jwk keys must be removed by deleting the DID itself" : did.keys.length === 1 ? "Cannot remove the only key of a DID" : "Remove key from DID",
                      children: /* @__PURE__ */ jsx8(Trash22, { className: "h-3 w-3 text-destructive" })
                    }
                  )
                ] })
              }
            ]
          }
        ),
        isWeb ? /* @__PURE__ */ jsx8(
          KeyPicker,
          {
            excludeIds: did.keys.map((k) => k.internal),
            value: newKeyId,
            onChange: setNewKeyId,
            disabled: addKey.isPending,
            onSubmit: () => addKey.mutate(newKeyId.trim()),
            isSubmitting: addKey.isPending
          }
        ) : /* @__PURE__ */ jsx8("p", { className: "mt-3 text-xs text-muted-foreground italic", children: "did:jwk DIDs hold a single key bound to their identifier. Create a new DID instead." })
      ] }),
      /* @__PURE__ */ jsx8("div", { children: /* @__PURE__ */ jsxs5("div", { className: "bg-sunken/40 rounded-xl border border-ink/5 overflow-hidden", children: [
        /* @__PURE__ */ jsxs5("div", { className: "flex items-center justify-between px-4 py-2 border-b border-ink/5 bg-ink/[0.02]", children: [
          /* @__PURE__ */ jsxs5("span", { className: "text-xs font-bold uppercase tracking-widest text-primary/60 flex items-center gap-2", children: [
            /* @__PURE__ */ jsx8(FileJson, { className: "h-3 w-3" }),
            "DID Document"
          ] }),
          /* @__PURE__ */ jsxs5(
            "button",
            {
              type: "button",
              onClick: handleCopy,
              className: "flex items-center gap-1.5 text-xs font-mono text-muted-foreground hover:text-primary transition-colors",
              children: [
                copied ? /* @__PURE__ */ jsx8(Check2, { className: "h-3 w-3 text-green-500" }) : /* @__PURE__ */ jsx8(Copy, { className: "h-3 w-3" }),
                copied ? "Copied!" : "Copy"
              ]
            }
          )
        ] }),
        /* @__PURE__ */ jsx8("div", { className: "p-4 overflow-x-auto", children: /* @__PURE__ */ jsx8("pre", { className: "font-mono text-xs text-muted-foreground/90 whitespace-pre-wrap break-all leading-relaxed", children: formattedDoc }) })
      ] }) })
    ] })
  ] });
};
var KeyPicker = ({
  excludeIds,
  value,
  onChange,
  disabled,
  onSubmit,
  isSubmitting
}) => {
  const { data: keysRes } = useQuery4({
    queryKey: WALLET_KEYS_QUERY_KEY,
    queryFn: () => customInstance("/wallet/keys", { method: "GET" })
  });
  const allKeys = keysRes?.status === 200 ? keysRes.data : [];
  const candidates = allKeys.filter((k) => !excludeIds.includes(k.id));
  return /* @__PURE__ */ jsxs5("div", { className: "mt-4 space-y-2", children: [
    /* @__PURE__ */ jsx8(Label, { className: "text-xs uppercase tracking-widest text-muted-foreground/70", children: "Attach existing key" }),
    /* @__PURE__ */ jsxs5("div", { className: "flex items-center gap-2", children: [
      /* @__PURE__ */ jsxs5(
        Select,
        {
          value,
          onValueChange: onChange,
          disabled: disabled || candidates.length === 0,
          children: [
            /* @__PURE__ */ jsx8(SelectTrigger, { className: "flex-1 font-mono text-xs", children: /* @__PURE__ */ jsx8(SelectValue, { placeholder: "\u2014 Choose a key \u2014" }) }),
            /* @__PURE__ */ jsx8(SelectContent, { children: candidates.map((k) => /* @__PURE__ */ jsx8(SelectItem, { value: k.id, className: "font-mono text-xs", children: k.alias ? `${k.alias} (${k.id})` : k.id }, k.id)) })
          ]
        }
      ),
      /* @__PURE__ */ jsxs5(Button, { size: "sm", disabled: !value || disabled, onClick: onSubmit, children: [
        isSubmitting ? /* @__PURE__ */ jsx8(Loader23, { className: "h-3 w-3 animate-spin" }) : /* @__PURE__ */ jsx8(Plus2, { className: "h-3 w-3" }),
        /* @__PURE__ */ jsx8("span", { className: "ml-1", children: "Add key" })
      ] })
    ] }),
    candidates.length === 0 && /* @__PURE__ */ jsx8("p", { className: "text-xs text-muted-foreground italic", children: "No spare keys available. Create one from the Keys tab." })
  ] });
};
var SERVICE_TYPES = ["AuthorizationServer", "CredentialIssuer", "FederatedCatalog"];
var NewDidDialog = ({ onCreated }) => {
  const [open, setOpen] = useState3(false);
  const [kind, setKind] = useState3("jwk");
  const [alias, setAlias] = useState3("");
  const [domain, setDomain] = useState3("");
  const [path, setPath] = useState3("");
  const [port, setPort] = useState3("");
  const [keysId, setKeysId] = useState3([]);
  const [services, setServices] = useState3([]);
  const { data: keysRes } = useQuery4({
    queryKey: WALLET_KEYS_QUERY_KEY,
    queryFn: () => customInstance("/wallet/keys", { method: "GET" })
  });
  const allKeys = keysRes?.status === 200 ? keysRes.data : [];
  const resetForm = () => {
    setKind("jwk");
    setAlias("");
    setDomain("");
    setPath("");
    setPort("");
    setKeysId([]);
    setServices([]);
  };
  const register = useMutation3({
    mutationFn: () => {
      const builder = kind === "web" ? {
        Web: {
          domain: domain.trim(),
          path: path.trim() ? path.trim() : null,
          port: port.trim() ? port.trim() : null
        }
      } : { Jwk: { pem: "" } };
      const cleanedServices = services.map((s) => ({
        ...s.id ? { id: s.id } : {},
        type: s.type.trim(),
        serviceEndpoint: s.serviceEndpoint.trim()
      })).filter((s) => s.type && s.serviceEndpoint);
      return customInstance("/wallet/did", {
        method: "POST",
        data: {
          builder,
          keys_id: keysId,
          alias,
          service: cleanedServices.length > 0 ? cleanedServices : null
        }
      });
    },
    onSuccess: async () => {
      toast3.success("DID registered");
      setOpen(false);
      resetForm();
      await onCreated();
    },
    onError: (err) => reportError(err, "Failed to register DID")
  });
  const toggleKey = (id) => {
    if (kind === "jwk") {
      setKeysId([id]);
    } else {
      setKeysId((curr) => curr.includes(id) ? curr.filter((k) => k !== id) : [...curr, id]);
    }
  };
  const addService = () => setServices((s) => [...s, { type: "", serviceEndpoint: "" }]);
  const removeService = (idx) => setServices((s) => s.filter((_, i) => i !== idx));
  const patchService = (idx, patch) => setServices((s) => s.map((v, i) => i === idx ? { ...v, ...patch } : v));
  const canSubmit = Boolean(alias.trim()) && keysId.length > 0 && (kind === "jwk" || domain.trim() !== "") && !register.isPending;
  return /* @__PURE__ */ jsxs5(Dialog, { open, onOpenChange: setOpen, children: [
    /* @__PURE__ */ jsx8(DialogTrigger, { asChild: true, children: /* @__PURE__ */ jsxs5(Button, { size: "sm", children: [
      /* @__PURE__ */ jsx8(Plus2, { className: "h-3 w-3" }),
      /* @__PURE__ */ jsx8("span", { className: "ml-1", children: "New DID" })
    ] }) }),
    /* @__PURE__ */ jsxs5(DialogContent, { className: "max-w-2xl max-h-[85vh] overflow-y-auto", children: [
      /* @__PURE__ */ jsxs5(DialogHeader, { children: [
        /* @__PURE__ */ jsx8(DialogTitle, { children: "Register a new DID" }),
        /* @__PURE__ */ jsx8(DialogDescription, { children: "Pick a method, attach an existing wallet key, optionally declare service endpoints." })
      ] }),
      /* @__PURE__ */ jsxs5("div", { className: "space-y-5", children: [
        /* @__PURE__ */ jsxs5("div", { className: "space-y-1", children: [
          /* @__PURE__ */ jsx8(Label, { className: "text-xs", children: "DID method" }),
          /* @__PURE__ */ jsxs5("div", { className: "flex gap-2", children: [
            /* @__PURE__ */ jsx8(
              Button,
              {
                size: "sm",
                variant: kind === "jwk" ? "default" : "outline",
                onClick: () => {
                  setKind("jwk");
                  setKeysId((curr) => curr.slice(0, 1));
                },
                children: "did:jwk"
              }
            ),
            /* @__PURE__ */ jsx8(
              Button,
              {
                size: "sm",
                variant: kind === "web" ? "default" : "outline",
                onClick: () => setKind("web"),
                children: "did:web"
              }
            )
          ] }),
          /* @__PURE__ */ jsx8("p", { className: "text-xs text-muted-foreground italic", children: kind === "jwk" ? "did:jwk derives the identifier from a single key's public material." : "did:web is hosted at the given URL; you can bind multiple keys." })
        ] }),
        /* @__PURE__ */ jsxs5("div", { className: "space-y-1", children: [
          /* @__PURE__ */ jsx8(Label, { className: "text-xs", children: "Alias" }),
          /* @__PURE__ */ jsx8(
            Input,
            {
              placeholder: kind === "jwk" ? "my-jwk-did" : "my-web-did",
              value: alias,
              onChange: (e) => setAlias(e.target.value)
            }
          )
        ] }),
        kind === "web" && /* @__PURE__ */ jsxs5(Fragment2, { children: [
          /* @__PURE__ */ jsxs5("div", { className: "space-y-1", children: [
            /* @__PURE__ */ jsx8(Label, { className: "text-xs", children: "Domain" }),
            /* @__PURE__ */ jsx8(
              Input,
              {
                placeholder: "example.com",
                value: domain,
                onChange: (e) => setDomain(e.target.value)
              }
            )
          ] }),
          /* @__PURE__ */ jsxs5("div", { className: "grid grid-cols-2 gap-3", children: [
            /* @__PURE__ */ jsxs5("div", { className: "space-y-1", children: [
              /* @__PURE__ */ jsx8(Label, { className: "text-xs", children: "Path (optional)" }),
              /* @__PURE__ */ jsx8(
                Input,
                {
                  placeholder: "agents/01",
                  value: path,
                  onChange: (e) => setPath(e.target.value)
                }
              )
            ] }),
            /* @__PURE__ */ jsxs5("div", { className: "space-y-1", children: [
              /* @__PURE__ */ jsx8(Label, { className: "text-xs", children: "Port (optional)" }),
              /* @__PURE__ */ jsx8(
                Input,
                {
                  placeholder: "8443",
                  value: port,
                  onChange: (e) => setPort(e.target.value)
                }
              )
            ] })
          ] })
        ] }),
        /* @__PURE__ */ jsxs5("div", { className: "space-y-1", children: [
          /* @__PURE__ */ jsx8(Label, { className: "text-xs", children: kind === "jwk" ? "Bind key (single)" : "Attach keys (one or more)" }),
          allKeys.length === 0 ? /* @__PURE__ */ jsx8("p", { className: "text-xs text-muted-foreground italic", children: "No keys available. Create one from the Keys tab." }) : /* @__PURE__ */ jsx8("div", { className: "space-y-1 max-h-40 overflow-y-auto border border-ink/10 rounded-lg p-2", children: allKeys.map((k) => /* @__PURE__ */ jsxs5(
            "label",
            {
              className: "flex items-center gap-2 p-1 text-xs cursor-pointer hover:bg-ink/5 rounded",
              children: [
                kind === "jwk" ? /* @__PURE__ */ jsx8(
                  "input",
                  {
                    type: "radio",
                    name: "did-keys",
                    checked: keysId.includes(k.id),
                    onChange: () => toggleKey(k.id),
                    className: "accent-primary h-3.5 w-3.5 [color-scheme:dark]"
                  }
                ) : /* @__PURE__ */ jsx8(
                  Checkbox,
                  {
                    checked: keysId.includes(k.id),
                    onCheckedChange: () => toggleKey(k.id)
                  }
                ),
                /* @__PURE__ */ jsx8("span", { className: "font-mono", children: k.alias || k.id }),
                /* @__PURE__ */ jsx8("span", { className: "font-mono text-xs text-muted-foreground truncate", children: k.id })
              ]
            },
            k.id
          )) })
        ] }),
        /* @__PURE__ */ jsxs5("div", { className: "space-y-2", children: [
          /* @__PURE__ */ jsxs5("div", { className: "flex items-center justify-between", children: [
            /* @__PURE__ */ jsx8(Label, { className: "text-xs", children: "Services (optional)" }),
            /* @__PURE__ */ jsxs5(Button, { size: "sm", variant: "ghost", onClick: addService, children: [
              /* @__PURE__ */ jsx8(Plus2, { className: "h-3 w-3" }),
              /* @__PURE__ */ jsx8("span", { className: "ml-1", children: "Add service" })
            ] })
          ] }),
          services.length === 0 ? /* @__PURE__ */ jsx8("p", { className: "text-xs text-muted-foreground italic", children: "Declare any service endpoint to expose in the DID document." }) : /* @__PURE__ */ jsx8("div", { className: "space-y-2", children: services.map((svc, idx) => /* @__PURE__ */ jsxs5(
            "div",
            {
              className: "border border-ink/10 rounded-lg p-3 space-y-2 bg-ink/[0.02]",
              children: [
                /* @__PURE__ */ jsxs5("div", { className: "flex items-center gap-2", children: [
                  /* @__PURE__ */ jsxs5("div", { className: "flex-1 space-y-1", children: [
                    /* @__PURE__ */ jsx8(Label, { className: "text-xs text-muted-foreground", children: "Type" }),
                    /* @__PURE__ */ jsx8(
                      Input,
                      {
                        list: `service-types-${idx}`,
                        placeholder: "AuthorizationServer / CredentialIssuer / ...",
                        value: svc.type,
                        onChange: (e) => patchService(idx, { type: e.target.value }),
                        className: "font-mono text-xs"
                      }
                    ),
                    /* @__PURE__ */ jsx8("datalist", { id: `service-types-${idx}`, children: SERVICE_TYPES.map((t) => /* @__PURE__ */ jsx8("option", { value: t }, t)) })
                  ] }),
                  /* @__PURE__ */ jsx8(
                    Button,
                    {
                      size: "sm",
                      variant: "ghost",
                      onClick: () => removeService(idx),
                      className: "mt-4",
                      children: /* @__PURE__ */ jsx8(Trash22, { className: "h-3 w-3 text-destructive" })
                    }
                  )
                ] }),
                /* @__PURE__ */ jsxs5("div", { className: "space-y-1", children: [
                  /* @__PURE__ */ jsx8(Label, { className: "text-xs text-muted-foreground", children: "Service Endpoint" }),
                  /* @__PURE__ */ jsx8(
                    Input,
                    {
                      placeholder: "https://example.com/oidc",
                      value: svc.serviceEndpoint,
                      onChange: (e) => patchService(idx, { serviceEndpoint: e.target.value })
                    }
                  )
                ] })
              ]
            },
            idx
          )) })
        ] })
      ] }),
      /* @__PURE__ */ jsxs5(DialogFooter, { children: [
        /* @__PURE__ */ jsx8(Button, { variant: "outline", onClick: () => setOpen(false), children: "Cancel" }),
        /* @__PURE__ */ jsxs5(Button, { disabled: !canSubmit, onClick: () => register.mutate(), children: [
          register.isPending ? /* @__PURE__ */ jsx8(Loader23, { className: "h-3 w-3 animate-spin mr-1" }) : /* @__PURE__ */ jsx8(Plus2, { className: "h-3 w-3 mr-1" }),
          "Register DID"
        ] })
      ] })
    ] })
  ] });
};

// src/wallet/WalletCredentialsView.tsx
import { useState as useState4 } from "react";
import { useQuery as useQuery5, useQueryClient as useQueryClient4 } from "@tanstack/react-query";
import {
  ArrowRight,
  Building2,
  Calendar,
  Check as Check3,
  ChevronDown,
  ChevronUp,
  FileJson as FileJson2,
  Fingerprint,
  Loader2 as Loader24,
  ShieldAlert,
  ShieldCheck,
  Sparkles,
  Trash2 as Trash23
} from "lucide-react";
import { toast as toast4 } from "sonner";
import { Fragment as Fragment3, jsx as jsx9, jsxs as jsxs6 } from "react/jsx-runtime";
var REGISTRATION_TYPES = [
  "gx:Eori",
  "gx:Euid",
  "gx:LeiCode",
  "gx:LocalRegistrationNumber",
  "gx:TaxId",
  "gx:VatId",
  "Eori",
  "TaxId",
  "VatId"
];
var LEGAL_PERSON_TYPES = ["gx:LegalPerson", "LegalPerson"];
var TC_TYPES = ["gx:TermsAndConditions", "TermsAndConditions"];
var LABEL_TYPES = ["gx:LabelCredential", "LabelCredential"];
var RECOMMENDED_REGISTRATION_VC = "gx_VatId_jwt_vc_json";
var RECOMMENDED_COMPLIANCE_VC = "gx_LabelCredential_jwt_vc_json";
function vcHasAny(vcs, targets) {
  return vcs.some((vc) => {
    const types = vc.parsed_document?.type || [];
    return types.some((t) => targets.includes(t));
  });
}
function WalletCredentialsView({
  showGaiaCompliance = true,
  requestVcHref = (rec) => `/authority/new?recommended=${encodeURIComponent(rec)}`
} = {}) {
  const queryClient = useQueryClient4();
  const {
    data: response,
    isLoading,
    error
  } = useQuery5({
    queryKey: ["wallet-vcs-custom"],
    queryFn: () => customInstance("/wallet/vcs", { method: "GET" })
  });
  const [isGenerating, setIsGenerating] = useState4(false);
  const credentials = response?.status === 200 ? response.data : [];
  const hasRegistration = vcHasAny(credentials, REGISTRATION_TYPES);
  const hasLegalPerson = vcHasAny(credentials, LEGAL_PERSON_TYPES);
  const hasTermsAndConditions = vcHasAny(credentials, TC_TYPES);
  const hasLabel = vcHasAny(credentials, LABEL_TYPES);
  const step = hasLabel ? 4 : hasRegistration && hasLegalPerson && hasTermsAndConditions ? 3 : hasRegistration ? 2 : 1;
  const handleGenerateGaia = async () => {
    setIsGenerating(true);
    try {
      await customInstance("/gaia/generate", { method: "POST" });
      toast4.success("Gaia-X self-issued credentials generated");
      await queryClient.invalidateQueries({ queryKey: ["wallet-vcs-custom"] });
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Failed to generate Gaia-X credentials";
      toast4.error(msg);
    } finally {
      setIsGenerating(false);
    }
  };
  const handleDeleteVc = async (id) => {
    try {
      await customInstance(`/wallet/credential/${encodeURIComponent(id)}`, { method: "DELETE" });
      toast4.success("Credential deleted");
      await queryClient.invalidateQueries({ queryKey: ["wallet-vcs-custom"] });
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Failed to delete credential";
      toast4.error(msg);
    }
  };
  if (isLoading) {
    return /* @__PURE__ */ jsx9(PageSection, { title: "Verifiable Credentials", children: /* @__PURE__ */ jsxs6("div", { className: "grid grid-cols-1 md:grid-cols-2 gap-6", children: [
      /* @__PURE__ */ jsx9(Skeleton, { className: "h-48 w-full rounded-xl" }),
      /* @__PURE__ */ jsx9(Skeleton, { className: "h-48 w-full rounded-xl" })
    ] }) });
  }
  if (error) {
    return /* @__PURE__ */ jsxs6("div", { className: "flex flex-col items-center justify-center py-20 text-destructive font-mono text-xs", children: [
      /* @__PURE__ */ jsx9(ShieldAlert, { className: "h-8 w-8 mb-2" }),
      "Error loading credentials"
    ] });
  }
  return /* @__PURE__ */ jsxs6("div", { className: "space-y-8 pb-20", children: [
    showGaiaCompliance && /* @__PURE__ */ jsxs6(PageSection, { title: "Gaia-X Compliance", children: [
      /* @__PURE__ */ jsx9(StepTrail, { step }),
      /* @__PURE__ */ jsx9(
        ComplianceStepCard,
        {
          step,
          isGenerating,
          onGenerate: handleGenerateGaia,
          requestVcHref,
          flags: { hasRegistration, hasLegalPerson, hasTermsAndConditions, hasLabel }
        }
      )
    ] }),
    /* @__PURE__ */ jsx9(PageSection, { title: "Verifiable Credentials", children: credentials.length === 0 ? /* @__PURE__ */ jsxs6("div", { className: "flex flex-col items-center justify-center py-20 text-muted-foreground border border-dashed border-ink/10 rounded-2xl bg-ink/2", children: [
      /* @__PURE__ */ jsx9(ShieldAlert, { className: "h-12 w-12 opacity-10 mb-4" }),
      /* @__PURE__ */ jsx9("p", { className: "text-sm font-medium", children: "No credentials found in this wallet." }),
      /* @__PURE__ */ jsx9("p", { className: "text-xs opacity-60", children: "Your claimed credentials will appear here." })
    ] }) : /* @__PURE__ */ jsx9("div", { className: "grid grid-cols-1 gap-6", children: credentials.map((vc, idx) => /* @__PURE__ */ jsx9(CredentialCard, { vc, onDelete: handleDeleteVc }, vc.id || idx)) }) })
  ] });
}
var CredentialCard = ({
  vc,
  onDelete
}) => {
  const [isExpanded, setIsExpanded] = useState4(false);
  const [isDeleting, setIsDeleting] = useState4(false);
  const parsed = vc.parsed_document || {};
  const issuerName = parsed.issuer?.name || parsed.issuer?.id || "Unknown Issuer";
  const types = (parsed.type || []).filter((t) => t !== "VerifiableCredential");
  const displayType = types.length > 0 ? types[types.length - 1] : "Credential";
  const addedRaw = vc.added_on ?? vc.addedOn ?? null;
  const addedDate = /* @__PURE__ */ jsx9(FormatDate, { date: addedRaw, format: DATE_FORMAT });
  const validRaw = vc.valid_until ?? parsed.validUntil ?? parsed.expirationDate;
  const validUntil = validRaw ? /* @__PURE__ */ jsx9(FormatDate, { date: validRaw, format: DATE_FORMAT }) : "Never";
  const handleDelete = async (e) => {
    e.stopPropagation();
    setIsDeleting(true);
    await onDelete(vc.id);
    setIsDeleting(false);
  };
  return /* @__PURE__ */ jsxs6(
    "div",
    {
      className: cn(
        "group relative overflow-hidden bg-ink/[0.03] border border-ink/10 rounded-2xl transition-all duration-300",
        isExpanded ? "border-primary/40 bg-ink/[0.06] ring-1 ring-primary/20" : "hover:border-ink/20 hover:bg-ink/[0.05]"
      ),
      children: [
        /* @__PURE__ */ jsxs6(
          "div",
          {
            onClick: () => setIsExpanded(!isExpanded),
            className: "cursor-pointer p-6 flex flex-col md:flex-row md:items-center justify-between gap-4",
            children: [
              /* @__PURE__ */ jsxs6("div", { className: "flex items-start gap-4", children: [
                /* @__PURE__ */ jsx9(
                  "div",
                  {
                    className: cn(
                      "p-3 rounded-xl transition-colors duration-300",
                      isExpanded ? "bg-primary/20 text-primary" : "bg-ink/5 text-muted-foreground group-hover:bg-ink/10"
                    ),
                    children: /* @__PURE__ */ jsx9(Fingerprint, { className: "h-6 w-6" })
                  }
                ),
                /* @__PURE__ */ jsxs6("div", { className: "space-y-1", children: [
                  /* @__PURE__ */ jsxs6("div", { className: "flex items-center gap-2", children: [
                    /* @__PURE__ */ jsx9("h4", { className: "font-bold text-lg tracking-tight", children: displayType }),
                    types.length > 1 && /* @__PURE__ */ jsxs6(Badge, { variant: "info", className: "text-xs h-4 py-0 font-mono opacity-60", children: [
                      "+",
                      types.length - 1,
                      " more"
                    ] })
                  ] }),
                  /* @__PURE__ */ jsxs6("div", { className: "flex flex-wrap items-center gap-y-1 gap-x-4 text-sm text-muted-foreground/80", children: [
                    /* @__PURE__ */ jsxs6("span", { className: "flex items-center gap-1.5", children: [
                      /* @__PURE__ */ jsx9(Building2, { className: "h-3.5 w-3.5 opacity-60" }),
                      issuerName
                    ] }),
                    /* @__PURE__ */ jsxs6("span", { className: "flex items-center gap-1.5 font-mono text-xs opacity-60", children: [
                      "ID: ",
                      vc.id?.substring(0, 16),
                      "..."
                    ] })
                  ] })
                ] })
              ] }),
              /* @__PURE__ */ jsxs6("div", { className: "flex items-center justify-between md:justify-end gap-6 border-t md:border-t-0 pt-4 md:pt-0 border-ink/5", children: [
                /* @__PURE__ */ jsxs6("div", { className: "flex items-center gap-4 text-xs text-muted-foreground/60", children: [
                  /* @__PURE__ */ jsxs6("div", { className: "flex flex-col items-end", children: [
                    /* @__PURE__ */ jsx9("span", { className: "uppercase text-xs font-bold tracking-widest opacity-40", children: "Added" }),
                    /* @__PURE__ */ jsxs6("span", { className: "flex items-center gap-1 mt-0.5", children: [
                      /* @__PURE__ */ jsx9(Calendar, { className: "h-3 w-3" }),
                      addedDate
                    ] })
                  ] }),
                  /* @__PURE__ */ jsxs6("div", { className: "flex flex-col items-end", children: [
                    /* @__PURE__ */ jsx9("span", { className: "uppercase text-xs font-bold tracking-widest opacity-40", children: "Expires" }),
                    /* @__PURE__ */ jsxs6("span", { className: "flex items-center gap-1 mt-0.5", children: [
                      /* @__PURE__ */ jsx9(ShieldCheck, { className: "h-3 w-3" }),
                      validUntil
                    ] })
                  ] })
                ] }),
                /* @__PURE__ */ jsx9("div", { className: "text-muted-foreground/40 group-hover:text-primary/60 transition-colors", children: isExpanded ? /* @__PURE__ */ jsx9(ChevronUp, { className: "h-5 w-5" }) : /* @__PURE__ */ jsx9(ChevronDown, { className: "h-5 w-5" }) })
              ] })
            ]
          }
        ),
        /* @__PURE__ */ jsx9(
          "div",
          {
            className: cn(
              "grid transition-all duration-500 ease-in-out",
              isExpanded ? "grid-rows-[1fr] border-t border-ink/10" : "grid-rows-[0fr]"
            ),
            children: /* @__PURE__ */ jsx9("div", { className: "overflow-hidden", children: /* @__PURE__ */ jsxs6("div", { className: "p-6 space-y-4", children: [
              /* @__PURE__ */ jsxs6("div", { className: "flex items-center justify-between", children: [
                /* @__PURE__ */ jsxs6("span", { className: "text-xs font-bold uppercase tracking-widest text-primary/60 flex items-center gap-2", children: [
                  /* @__PURE__ */ jsx9(FileJson2, { className: "h-3 w-3" }),
                  "Raw Credential Document"
                ] }),
                /* @__PURE__ */ jsxs6("div", { className: "flex items-center gap-2", children: [
                  /* @__PURE__ */ jsxs6(
                    Button,
                    {
                      variant: "ghost",
                      size: "sm",
                      className: "h-7 text-xs font-mono hover:bg-destructive/10 hover:text-destructive",
                      onClick: handleDelete,
                      disabled: isDeleting,
                      children: [
                        isDeleting ? /* @__PURE__ */ jsx9(Loader24, { className: "h-3 w-3 mr-1 animate-spin" }) : /* @__PURE__ */ jsx9(Trash23, { className: "h-3 w-3 mr-1" }),
                        "Delete"
                      ]
                    }
                  ),
                  /* @__PURE__ */ jsx9(
                    Button,
                    {
                      variant: "ghost",
                      size: "sm",
                      className: "h-7 text-xs font-mono hover:bg-primary/10 hover:text-primary",
                      onClick: (e) => {
                        e.stopPropagation();
                        navigator.clipboard.writeText(JSON.stringify(vc, null, 2));
                      },
                      children: "Copy JSON"
                    }
                  )
                ] })
              ] }),
              /* @__PURE__ */ jsx9("div", { className: "relative rounded-xl bg-sunken/40 border border-ink/5 p-4 font-mono text-xs leading-relaxed text-muted-foreground/90 overflow-x-auto shadow-inner", children: /* @__PURE__ */ jsx9("pre", { className: "whitespace-pre-wrap break-all", children: JSON.stringify(vc, null, 2) }) })
            ] }) })
          }
        )
      ]
    }
  );
};
var ComplianceStepCard = ({
  step,
  isGenerating,
  onGenerate,
  requestVcHref,
  flags
}) => {
  const wrapper = cn(
    "relative overflow-hidden p-6 rounded-2xl border transition-all duration-300",
    step === 4 ? "bg-green-500/10 border-green-500/20 shadow-lg" : step === 1 ? "bg-ink/2 border-ink/5 opacity-90" : "bg-primary/5 border-primary/20 shadow-lg shadow-primary/5"
  );
  return /* @__PURE__ */ jsxs6("div", { className: wrapper, children: [
    /* @__PURE__ */ jsxs6("div", { className: "flex flex-col md:flex-row md:items-center justify-between gap-6", children: [
      /* @__PURE__ */ jsxs6("div", { className: "space-y-2", children: [
        /* @__PURE__ */ jsxs6("div", { className: "flex items-center gap-2", children: [
          /* @__PURE__ */ jsx9(
            ShieldCheck,
            {
              className: cn(
                "h-5 w-5",
                step === 4 ? "text-green-500" : step === 1 ? "text-muted-foreground" : "text-primary"
              )
            }
          ),
          /* @__PURE__ */ jsx9("h3", { className: "font-semibold text-lg", children: stepTitle(step) }),
          /* @__PURE__ */ jsxs6(Badge, { variant: "info", className: "font-mono text-xs", children: [
            "Step ",
            step,
            " of 4"
          ] })
        ] }),
        /* @__PURE__ */ jsx9("p", { className: "text-sm text-muted-foreground max-w-xl", children: stepDescription(step) })
      ] }),
      /* @__PURE__ */ jsx9(
        StepCta,
        {
          step,
          isGenerating,
          onGenerate,
          requestVcHref
        }
      )
    ] }),
    step === 1 && /* @__PURE__ */ jsx9(
      Hint,
      {
        tone: "amber",
        text: "Request a registration credential (VatId, TaxId or LeiCode) from a legal authority."
      }
    ),
    step === 2 && !flags.hasLegalPerson && !flags.hasTermsAndConditions && /* @__PURE__ */ jsx9(
      Hint,
      {
        tone: "primary",
        text: "The wallet will self-issue gx:LegalPerson and gx:TermsAndConditions on top of your registration credential."
      }
    ),
    step === 3 && /* @__PURE__ */ jsx9(
      Hint,
      {
        tone: "primary",
        text: "Ask a clearing house to issue a LabelCredential signing your self-attested data."
      }
    )
  ] });
};
var StepCta = ({
  step,
  isGenerating,
  onGenerate,
  requestVcHref
}) => {
  const Link = useLink();
  if (step === 1) {
    return /* @__PURE__ */ jsx9(Link, { to: requestVcHref(RECOMMENDED_REGISTRATION_VC), children: /* @__PURE__ */ jsxs6(Button, { className: "md:min-w-[220px]", children: [
      "Request VC",
      /* @__PURE__ */ jsx9(ArrowRight, { className: "ml-2 h-4 w-4" })
    ] }) });
  }
  if (step === 2) {
    return /* @__PURE__ */ jsx9(Button, { disabled: isGenerating, onClick: onGenerate, className: "md:min-w-[220px]", children: isGenerating ? /* @__PURE__ */ jsxs6(Fragment3, { children: [
      /* @__PURE__ */ jsx9(Loader24, { className: "mr-2 h-4 w-4 animate-spin" }),
      "Generating..."
    ] }) : /* @__PURE__ */ jsxs6(Fragment3, { children: [
      /* @__PURE__ */ jsx9(Sparkles, { className: "mr-2 h-4 w-4" }),
      "Auto-generate VCs"
    ] }) });
  }
  if (step === 3) {
    return /* @__PURE__ */ jsx9(Link, { to: requestVcHref(RECOMMENDED_COMPLIANCE_VC), children: /* @__PURE__ */ jsxs6(Button, { className: "md:min-w-[220px]", children: [
      "Request Compliance",
      /* @__PURE__ */ jsx9(ArrowRight, { className: "ml-2 h-4 w-4" })
    ] }) });
  }
  return /* @__PURE__ */ jsxs6(Button, { variant: "outline", disabled: true, className: "md:min-w-[220px]", children: [
    /* @__PURE__ */ jsx9(Check3, { className: "mr-2 h-4 w-4 text-green-500" }),
    "Compliance Active"
  ] });
};
function stepTitle(step) {
  switch (step) {
    case 1:
      return "Get your legal identity credential";
    case 2:
      return "Self-issue Gaia-X base credentials";
    case 3:
      return "Request the compliance credential";
    case 4:
      return "You are Gaia-X compliant";
  }
}
function stepDescription(step) {
  switch (step) {
    case 1:
      return "Pick a registration credential (VatId, TaxId, LEI, EORI, ...) from a trusted authority. This anchors your legal identity.";
    case 2:
      return "Generate self-signed gx:LegalPerson and gx:TermsAndConditions credentials linked to your registration credential.";
    case 3:
      return "Submit your credentials to a Gaia-X Clearing House to receive your LabelCredential.";
    case 4:
      return "Your wallet holds a valid Gaia-X LabelCredential and is ready to participate in Gaia-X data spaces.";
  }
}
var Hint = ({ tone, text }) => {
  const cls = tone === "amber" ? "bg-warn-500/10 border-warn-500/30 text-warn-700 dark:text-warn-300" : "bg-primary/10 border-primary/20 text-primary";
  return /* @__PURE__ */ jsxs6("div", { className: cn("mt-4 p-3 rounded-lg text-sm flex items-center gap-2 border", cls), children: [
    /* @__PURE__ */ jsx9(ShieldAlert, { className: "h-4 w-4 shrink-0" }),
    text
  ] });
};
var StepTrail = ({ step }) => {
  const items = [
    { n: 1, label: "Registration" },
    { n: 2, label: "Self-issued" },
    { n: 3, label: "Compliance" },
    { n: 4, label: "Compliant" }
  ];
  return /* @__PURE__ */ jsx9("div", { className: "mb-3 flex items-center gap-2", children: items.map((it, idx) => {
    const done = it.n < step;
    const current = it.n === step;
    return /* @__PURE__ */ jsxs6("div", { className: "flex items-center gap-2", children: [
      /* @__PURE__ */ jsx9(
        "div",
        {
          className: cn(
            "h-6 w-6 rounded-full flex items-center justify-center text-xs font-bold border",
            done ? "bg-green-500/20 border-green-500/40 text-green-500" : current ? "bg-primary/20 border-primary/40 text-primary" : "bg-ink/5 border-ink/10 text-muted-foreground"
          ),
          children: done ? /* @__PURE__ */ jsx9(Check3, { className: "h-3 w-3" }) : it.n
        }
      ),
      /* @__PURE__ */ jsx9(
        "span",
        {
          className: cn(
            "text-xs uppercase tracking-widest",
            current ? "text-primary font-bold" : "text-muted-foreground"
          ),
          children: it.label
        }
      ),
      idx < items.length - 1 && /* @__PURE__ */ jsx9("div", { className: "h-px w-6 bg-ink/10", "aria-hidden": true })
    ] }, it.n);
  }) });
};

// src/wallet/WalletOidc4vpView.tsx
import { useState as useState5 } from "react";
import { useMutation as useMutation4 } from "@tanstack/react-query";
import { Loader2 as Loader25, ShieldCheck as ShieldCheck2 } from "lucide-react";
import { toast as toast5 } from "sonner";
import { jsx as jsx10, jsxs as jsxs7 } from "react/jsx-runtime";
function WalletOidc4vpView() {
  const [uri, setUri] = useState5("");
  const {
    mutateAsync: processVp,
    isPending,
    isSuccess,
    error
  } = useMutation4({
    mutationFn: (data) => customInstance("/wallet/oid4vp", { method: "POST", data }),
    onSuccess: () => toast5.success("Presentation submitted successfully"),
    onError: (err) => {
      const msg = err instanceof Error ? err.message : "Failed to process presentation request";
      toast5.error(msg);
    }
  });
  const handleProcess = async (e) => {
    e.preventDefault();
    if (!uri) return;
    try {
      await processVp({ uri });
      setUri("");
    } catch {
    }
  };
  return /* @__PURE__ */ jsx10(PageSection, { title: "Present credentials (OID4VP)", children: /* @__PURE__ */ jsxs7(
    "form",
    {
      onSubmit: handleProcess,
      className: "max-w-2xl space-y-4 rounded-lg border border-ink/10 p-5",
      children: [
        /* @__PURE__ */ jsxs7("div", { className: "flex items-start gap-3", children: [
          /* @__PURE__ */ jsx10(ShieldCheck2, { className: "mt-0.5 h-5 w-5 shrink-0 text-ink/40" }),
          /* @__PURE__ */ jsx10("p", { className: "text-sm text-ink/70", children: "Paste the OID4VP request URI a verifier gave you to present credentials from this wallet." })
        ] }),
        /* @__PURE__ */ jsxs7("div", { className: "space-y-1.5", children: [
          /* @__PURE__ */ jsx10(Label, { htmlFor: "oidc4vp-uri", children: "Request URI" }),
          /* @__PURE__ */ jsx10(
            Input,
            {
              id: "oidc4vp-uri",
              placeholder: "openid-vc://\u2026",
              value: uri,
              onChange: (e) => setUri(e.target.value),
              className: "font-mono text-xs"
            }
          )
        ] }),
        /* @__PURE__ */ jsxs7("div", { className: "flex items-center gap-3", children: [
          /* @__PURE__ */ jsxs7(Button, { type: "submit", disabled: isPending || !uri, children: [
            isPending && /* @__PURE__ */ jsx10(Loader25, { className: "animate-spin" }),
            "Present credentials"
          ] }),
          isSuccess && /* @__PURE__ */ jsx10("span", { className: "text-sm text-success-700 dark:text-success-300", children: "Presentation sent to the verifier." }),
          !!error && /* @__PURE__ */ jsx10("span", { className: "text-sm text-danger-700 dark:text-danger-300", children: "The presentation request could not be processed." })
        ] })
      ]
    }
  ) });
}

// src/wallet/WalletOidc4vciView.tsx
import { useState as useState6 } from "react";
import { useMutation as useMutation5, useQueryClient as useQueryClient5 } from "@tanstack/react-query";
import { FileText, Loader2 as Loader26 } from "lucide-react";
import { toast as toast6 } from "sonner";
import { jsx as jsx11, jsxs as jsxs8 } from "react/jsx-runtime";
function WalletOidc4vciView() {
  const [uri, setUri] = useState6("");
  const queryClient = useQueryClient5();
  const {
    mutateAsync: processVci,
    isPending,
    isSuccess,
    error
  } = useMutation5({
    mutationFn: (data) => customInstance("/wallet/oid4vci", { method: "POST", data }),
    onSuccess: async () => {
      toast6.success("Credential issued successfully");
      await queryClient.invalidateQueries({ queryKey: ["wallet-vcs-custom"] });
    },
    onError: (err) => {
      const msg = err instanceof Error ? err.message : "Failed to process credential offer";
      toast6.error(msg);
    }
  });
  const handleProcess = async (e) => {
    e.preventDefault();
    if (!uri) return;
    try {
      await processVci({ uri });
      setUri("");
    } catch {
    }
  };
  return /* @__PURE__ */ jsx11(PageSection, { title: "Receive a credential (OID4VCI)", children: /* @__PURE__ */ jsxs8(
    "form",
    {
      onSubmit: handleProcess,
      className: "max-w-2xl space-y-4 rounded-lg border border-ink/10 p-5",
      children: [
        /* @__PURE__ */ jsxs8("div", { className: "flex items-start gap-3", children: [
          /* @__PURE__ */ jsx11(FileText, { className: "mt-0.5 h-5 w-5 shrink-0 text-ink/40" }),
          /* @__PURE__ */ jsx11("p", { className: "text-sm text-ink/70", children: "Paste the credential offer URI an issuer gave you to add the credential to this wallet." })
        ] }),
        /* @__PURE__ */ jsxs8("div", { className: "space-y-1.5", children: [
          /* @__PURE__ */ jsx11(Label, { htmlFor: "oidc4vci-uri", children: "Credential offer URI" }),
          /* @__PURE__ */ jsx11(
            Input,
            {
              id: "oidc4vci-uri",
              placeholder: "openid-credential-offer://\u2026",
              value: uri,
              onChange: (e) => setUri(e.target.value),
              className: "font-mono text-xs"
            }
          )
        ] }),
        /* @__PURE__ */ jsxs8("div", { className: "flex items-center gap-3", children: [
          /* @__PURE__ */ jsxs8(Button, { type: "submit", disabled: isPending || !uri, children: [
            isPending && /* @__PURE__ */ jsx11(Loader26, { className: "animate-spin" }),
            "Request credential"
          ] }),
          isSuccess && /* @__PURE__ */ jsx11("span", { className: "text-sm text-success-700 dark:text-success-300", children: "Credential added to the wallet." }),
          !!error && /* @__PURE__ */ jsx11("span", { className: "text-sm text-danger-700 dark:text-danger-300", children: "The credential offer could not be processed." })
        ] })
      ]
    }
  ) });
}

export {
  Checkbox,
  ConfirmDialog,
  Label,
  Textarea,
  WalletLayout,
  WalletInfoView,
  WALLET_KEYS_QUERY_KEY,
  WalletKeysView,
  WalletDidView,
  WalletCredentialsView,
  WalletOidc4vpView,
  WalletOidc4vciView
};
//# sourceMappingURL=chunk-X4X3RSGO.js.map