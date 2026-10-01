import { ContentCopy } from '@mui/icons-material'
import {
  Alert,
  Box,
  CircularProgress,
  IconButton,
  List,
  ListItem,
  ListItemText,
  Snackbar,
  TextField,
  Tooltip,
  Typography,
} from '@mui/material'
import { useLockFn } from 'ahooks'
import { useImperativeHandle, useState, type Ref } from 'react'
import { useTranslation } from 'react-i18next'

import { BaseDialog, DialogRef, Switch } from '@/components/base'
import { useVerge } from '@/hooks/use-verge'
import { getMcpServerRunning } from '@/services/cmds'
import { showNotice } from '@/services/notice-service'

const DEFAULT_MCP_PORT = 9898

export function McpServerViewer({ ref }: { ref?: Ref<DialogRef> }) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const [copySuccess, setCopySuccess] = useState<null | string>(null)
  const [isSaving, setIsSaving] = useState(false)
  const [isRunning, setIsRunning] = useState(false)

  const { verge, patchVerge } = useVerge()
  const [enabled, setEnabled] = useState(verge?.enable_mcp_server ?? false)
  const [allowMutations, setAllowMutations] = useState(
    verge?.mcp_server_allow_mutations ?? false,
  )
  const [port, setPort] = useState(
    String(verge?.mcp_server_port ?? DEFAULT_MCP_PORT),
  )
  const token = verge?.mcp_server_token ?? ''

  useImperativeHandle(ref, () => ({
    open: async () => {
      setOpen(true)
      setEnabled(verge?.enable_mcp_server ?? false)
      setAllowMutations(verge?.mcp_server_allow_mutations ?? false)
      setPort(String(verge?.mcp_server_port ?? DEFAULT_MCP_PORT))
      setIsRunning(await getMcpServerRunning())
    },
    close: () => setOpen(false),
  }))

  const onSave = useLockFn(async () => {
    try {
      setIsSaving(true)

      const portNum = parseInt(port, 10)
      const validPort = !isNaN(portNum) && portNum > 0 && portNum < 65536

      await patchVerge({
        enable_mcp_server: enabled,
        mcp_server_port: validPort ? portNum : DEFAULT_MCP_PORT,
        mcp_server_allow_mutations: allowMutations,
      })

      showNotice.success('shared.feedback.notifications.common.saveSuccess')
      setOpen(false)
    } catch (err) {
      showNotice.error(
        'shared.feedback.notifications.common.saveFailed',
        err,
        4000,
      )
    } finally {
      setIsSaving(false)
    }
  })

  const handleCopy = useLockFn(async (text: string, type: string) => {
    try {
      await navigator.clipboard.writeText(text)
      setCopySuccess(type)
      setTimeout(() => setCopySuccess(null), 2000)
    } catch {
      showNotice.error('settings.sections.mcpServer.messages.copyFailed')
    }
  })

  return (
    <BaseDialog
      open={open}
      title={t('settings.sections.mcpServer.title')}
      contentSx={{ width: 420 }}
      okBtn={
        isSaving ? (
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <CircularProgress size={16} color="inherit" />
            {t('shared.statuses.saving')}
          </Box>
        ) : (
          t('shared.actions.save')
        )
      }
      cancelBtn={t('shared.actions.cancel')}
      onClose={() => setOpen(false)}
      onCancel={() => setOpen(false)}
      onOk={onSave}
    >
      <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
        {t('settings.sections.mcpServer.description')}
      </Typography>
      <Typography
        variant="caption"
        color={isRunning ? 'success.main' : 'text.disabled'}
        sx={{ display: 'block', mb: 1 }}
      >
        {t(
          isRunning
            ? 'settings.sections.mcpServer.status.running'
            : 'settings.sections.mcpServer.status.stopped',
        )}
      </Typography>

      <List>
        <ListItem
          sx={{
            padding: '5px 2px',
            display: 'flex',
            justifyContent: 'space-between',
          }}
        >
          <ListItemText
            primary={t('settings.sections.mcpServer.fields.enable')}
          />
          <Switch
            edge="end"
            checked={enabled}
            onChange={(e) => setEnabled(e.target.checked)}
            disabled={isSaving}
          />
        </ListItem>

        <ListItem
          sx={{
            padding: '5px 2px',
            display: 'flex',
            justifyContent: 'space-between',
          }}
        >
          <ListItemText
            primary={t('settings.sections.mcpServer.fields.port')}
          />
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <TextField
              size="small"
              sx={{
                width: 150,
                opacity: enabled ? 1 : 0.5,
                pointerEvents: enabled ? 'auto' : 'none',
              }}
              value={port}
              placeholder={t('settings.sections.mcpServer.placeholders.port')}
              onChange={(e) => setPort(e.target.value)}
              disabled={isSaving || !enabled}
              type="number"
              slotProps={{ htmlInput: { min: 1, max: 65535 } }}
            />
          </Box>
        </ListItem>

        <ListItem
          sx={{
            padding: '5px 2px',
            display: 'flex',
            justifyContent: 'space-between',
          }}
        >
          <ListItemText
            primary={t('settings.sections.mcpServer.fields.token')}
            secondary={t('settings.sections.mcpServer.hints.tokenGenerated')}
          />
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <TextField
              size="small"
              sx={{ width: 150 }}
              value={token}
              placeholder={t(
                'settings.sections.mcpServer.placeholders.tokenPending',
              )}
              slotProps={{ input: { readOnly: true } }}
              disabled={!enabled}
            />
            <Tooltip title={t('settings.sections.mcpServer.tooltips.copy')}>
              <span>
                <IconButton
                  size="small"
                  onClick={() => handleCopy(token, 'token')}
                  color="primary"
                  disabled={!token}
                >
                  <ContentCopy fontSize="small" />
                </IconButton>
              </span>
            </Tooltip>
          </Box>
        </ListItem>

        <ListItem
          sx={{
            padding: '5px 2px',
            display: 'flex',
            justifyContent: 'space-between',
          }}
        >
          <ListItemText
            primary={t('settings.sections.mcpServer.fields.allowMutations')}
            secondary={t('settings.sections.mcpServer.hints.allowMutations')}
          />
          <Switch
            edge="end"
            checked={allowMutations}
            onChange={(e) => setAllowMutations(e.target.checked)}
            disabled={isSaving || !enabled}
          />
        </ListItem>
      </List>

      <Snackbar
        open={copySuccess !== null}
        autoHideDuration={2000}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
      >
        <Alert severity="success">
          {t('settings.sections.mcpServer.messages.tokenCopied')}
        </Alert>
      </Snackbar>
    </BaseDialog>
  )
}
